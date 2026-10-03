# PLAN-M7 — Ses yığını kararı: seçenekler (onay bekliyor)

**Tarih:** 2026-10-03 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25 · **Durum:** karar yok; kullanıcı onayı olmadan ses görevi açılmaz.

Bu kayıt bir davranış dilimi değildir; [AGENTS.md](../AGENTS.md)'nin "teknolojiyi ihtiyaç ve kaynak incelemesi ortaya çıkmadan seçme" kuralı gereği, ses bölümüne girmeden önceki kaynak incelemesidir ve [render yığını kararının](render-backend-decision.md) biçimini izler. Uygulama kodu eklenmedi.

## Referansın gerçekte kullandığı yığın

| Katman | Referansta (pinned 26.2 kaynağı ve `runtime/libs`) |
| --- | --- |
| Ses çıkışı | **OpenAL** (LWJGL `lwjgl-openal` 3.4.1; native'ler OpenAL Soft: linux, macOS x64/arm64, Windows x64/arm64/x86). `Library` aygıtı `ALC10.alcOpenDevice` ile açar, `alcCreateContext` (HRTF `ALC_SOFT_HRTF` varsa isteğe bağlı) kurar |
| Gerekli OpenAL uzantıları | `AL_EXT_source_distance_model` (yoksa istisna), `AL_EXT_LINEAR_DISTANCE` (yoksa istisna), `ALC_EXT_disconnect`, `ALC_SOFT_system_events` (aygıt değişimi) |
| Kanal modeli | 30 durağan + akış kanalı (`Library.ChannelPool`); her kanal bir OpenAL kaynağı: konum, perde (`AL_PITCH`), ses (`AL_GAIN`), döngü, göreli konum, **doğrusal mesafe sönümü** (`AL_LINEAR_DISTANCE_CLAMPED` = 53251, `AL_MAX_DISTANCE`, referans mesafesi 1, rolloff 0), kuyruk tamponları ile akış |
| Ses çözümü | **JOrbis 0.0.17** (`jorbis-0.0.17.jar`, libvorbis'in saf Java çözücüsü): `JOrbisAudioStream` Ogg sayfa/paket okur, `DspState`/`Block` ile sentezler, 16 bit işaretli little-endian PCM üretir. (`lwjgl-stb` yüklü ama sesin çözümünde kullanılmıyor.) |
| Seçim ve zamanlama | `SoundManager` (`sounds.json`, `WeighedSoundEvents`, ağırlıklı seçim), `SoundEngine`/`SoundEngineExecutor` (ayrı ses iş parçacığı), `ChannelAccess`, `MusicManager` |

Yani ses iki ayrı sorudur: (a) *hangi ses, hangi anda, hangi ses/perde/konumla* çalınır (saf mantık, Java'ya karşı karşılaştırılabilir) ve (b) *duyulan çıkış* (OpenAL kaynağı ve miks).

## Neyin karşılaştırılabileceği

- (a) `sounds.json` çözümü, ağırlıklı seçimin rastgele sayı akışı, `SoundEngine`'in kanala verdiği parametreler (konum, perde, ses, mesafe, döngü, durdurma), kategoriler ve duraklatma: oracle ile `client-parity` olabilir (headless; `SoundEngine` yerine kanal arayüzünü yakalayan sahte).
- (b) Çözülen PCM, JOrbis çıktısıyla örnek örnek karşılaştırılabilir (aynı `.ogg`, aynı 16 bit biçim). Duyulan miks aynı OpenAL Soft sürümü kullanılmadıkça bit eşitliği iddia edilemez; yakalama yöntemi ayrı kayıttır ([P20](roadmap.md): "aynı ses dosyasını seçmek duyulan eşitlik değildir").

## Aday yığınlar

Sürüm ve tarihler crates.io'dan 2026-10-03'te okundu.

### A — OpenAL Soft'a doğrudan bağlanma, referansın karşılığı

- Çıkış: OpenAL Soft'u dinamik yükleyerek (ör. `libloading` 0.9, MIT/Apache; sistemde `libopenal`) ince bir FFI katmanı; OpenAL sarmalayıcı crate'leri eski veya çok küçük (`alto` 3.0.4, 2019; `openal-sys` 1.16.0, 2016; `openal-soft-sys` 0.1.0, 2025, 393 indirme).
- Artı: kanal parametreleri referansla **aynı çağrılar** (`alSourcef(AL_PITCH)`, `AL_LINEAR_DISTANCE_CLAMPED`…); HRTF, aygıt listesi ve ayrılma olayları aynı uzantılarla; mesafe/yön modeli bizim yazdığımız bir şey değil → fark kaynağı gizlenmez (render kararındaki gerekçe).
- Eksi: OpenAL Soft her platformda kurulu değil (Windows'ta yok; macOS'un sistem OpenAL'ı Apple'ındır, OpenAL Soft değildir ve uzantıları farklı olabilir); paketlemek LGPL dosyasını dağıtmak demektir; "kurulum bir şey indirmez" kuralı kullanıcıya bir kurulum adımı yükler. Referans bunu native jar'larıyla çözüyor.

### B — `cpal` ile aygıt çıkışı + kendi miksimiz (OpenAL modelinin yeniden yazımı)

- `cpal` 0.18.2 (Apache-2.0): CoreAudio/WASAPI/ALSA gibi arka uçlar, dinamik OpenAL gerekmez.
- Artı: tek bağımlılık, her platformda çalışır, ayak izi küçük.
- Eksi: doğrusal sönüm, perde (yeniden örnekleme), panlama (OpenAL Soft'un varsayılan yerleştirmesi, isteğe bağlı HRTF), kaynak sayısı ve gecikme davranışını biz yazarız; "duyulan çıkış" referansla aynı olmaz, yalnız mantık eşit olur. HRTF ayarı karşılıksız kalır.

### C — Üst düzey ses kütüphanesi (`rodio` 0.22.2, `kira` 0.12.5)

- Artı: hızlı sonuç, karışım ve döngü hazır.
- Eksi: sönüm/perde/konum modeli kütüphanenin; referansın kanal sözleşmesi (kuyruk tamponları, kanal havuzu, `AL_*` parametreleri) ile örtüşmez. Aygıt tarafında `cpal` kullanır, yani B'nin üstüne soyutlama ekler. Bu projenin "çerçeve ekleme" tutumuyla da uyumsuz.

### Ogg/Vorbis çözücü adayları

| Aday | Not |
| --- | --- |
| `lewton` 0.10.2 (MIT/Apache-2.0, saf Rust) | Son sürüm 2021; çözücü tam; çıktısı JOrbis ile **örnek örnek karşılaştırılır** (yuvarlama/kırpma farkı olursa tolerans ya da kendi 16 bit dönüşümü) |
| `symphonia` 0.6.1 (`symphonia-codec-vorbis` + `symphonia-format-ogg`) | Aktif bakım, saf Rust; çok daha büyük yüzey (kullanılmayan codec'ler kapatılabilir); aynı karşılaştırma gerekir |
| libvorbis/stb_vorbis bağlama | C bağımlılığı; referans JOrbis kullandığından bit eşitliği için de en yakın aday değildir |
| JOrbis'in Rust'a çevirisi | Referansın aynısı, bit eşitliği en olası; ancak büyük bir port (yüzlerce satır DSP) ve bakım yükü |

## Öneri

1. **Çıkış: A ile B'yi ayır, ikisini de aynı kanal arayüzüne bağla.** Mantık katmanı (seçim, kanal parametreleri, kategori, duraklatma) **arayüzden önce** yazılıp Java'ya karşı `client-parity` olur; bu iş çıkış seçimini beklemez ve bağımlılık gerektirmez. Çıkış için başlangıçta **B (`cpal`)**: her platformda kurulumsuz çalışır ve mantık eşitliğini bozmaz; "duyulan eşitlik" iddiası yapılmaz, A ise ileride (P26 dağıtım matrisi) aynı arayüzün ikinci uygulaması olarak eklenebilir.
2. **Çözücü: `lewton`**, ilk görevde JOrbis çıktısıyla örnek karşılaştırması yapılır; fark çıkarsa `symphonia` denenir, ikisi de tutmazsa JOrbis portu kararı yeniden açılır.
3. Üretim bağımlılığı yalnız ses çıkışı ve çözücü içindir (`cpal` ve `lewton`); başka çerçeve eklenmez.

## Görev dilimleri (onay sonrası açılır)

| Kimlik (öneri) | Kapsam |
| --- | --- |
| T60 | `sounds.json` çözümü, `WeighedSoundEvents` ve ağırlıklı seçim (oracle, `client-parity`; bağımlılık yok) |
| T61 | `Sound`, `SoundEntity`, `StopSound` paketleri ve `SoundEngine` kanal parametreleri (oracle: kanalı yakalayan sahte) |
| T62 | Ogg/Vorbis çözümü (`lewton`; JOrbis ile PCM karşılaştırması) |
| T63 | Ses çıkışı (`cpal`): kanal havuzu, doğrusal mesafe sönümü, perde, döngü, akış; duyulan çıktı yakalama notu |
| T64 | Oyun tetikleri: adım, kırma/koyma, eşya, hasar, ortam, yağmur; müzik (`MusicManager`) |
| T65 | Ses ayarları ekranı ve kategori kaydırıcıları, duraklatmada devam eden kategoriler |

## Kullanıcıya sorulacak karar

Çıkış yığını A, B ya da C mi; çözücü `lewton` mu `symphonia` mı. Onaydan sonra bu belge "karar" belgesine çevrilir, ilgili görevler panoya açılır ve `Cargo.toml`'a bağımlılık o görevde eklenir.
