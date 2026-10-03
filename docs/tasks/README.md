# Görev panosu ve oturum protokolü

Her görev **tek oturum, tek commit**tir ve sıfır bağlamla başlanabilecek kadar kendi kendine yeterlidir. Oturum bitince bağlam sıfırlanabilir; sürekliliği bu pano ve görev dosyalarındaki devir notları taşır. Uzun vadeli kapsam kaydı [kalan işlerde](../remaining-work.md), bölüm/kabul kuralı [roadmap](../roadmap.md)tedir; ikisi de yalnız görev isterse okunur.

## Oturum protokolü

1. Aşağıdaki tabloda bağımlılıkları `bitti` olan ilk `açık` görevi seç (kullanıcı kimlik verdiyse onu).
2. Yalnız o görev dosyasını ve onun **Oku** listesini oku. Başka belge, alt ajan veya plan modu gerekmez; arama `grep -n`/`find` ile doğrudan yapılır.
3. Java sınıfını satır satır oku (`.data/minecraft/source/…`), oracle senaryosunu yaz, Rust'ı yaz. İterasyonda: `cargo build -p <crate> 2>&1 | grep -E '^(error|warning)' -A6 | head -60`, `cargo test -p <crate> <filtre>`, `python3 tools/voxel.py parity --family gui`.
4. Commit öncesi **bir kez** `python3 tools/voxel.py check` (oracle kaynağı veya `cases.py` değiştiyse `--capture`). `CHECK OK` görmeden commit yok; başarısızsa yalnız basılan hatayı düzelt.
5. Belge: görev dosyasının **Belgeler** satırı neyi güncelleyeceğini söyler; gui operation'ı eklediysen `docs/gui-oracle.md` tablosuna satır ekle. `python3 tools/voxel.py check` belge indeksini ve hücre sınırını da denetler.
6. Görev dosyasının sonuna **Devir notu** yaz (≤10 satır: yapılan, Java'dan sapma, açık kalan), bu tabloda durumu `bitti (<kısa hash>)` yap, tek commit at (Türkçe mesaj, **AI imzası/Co-Authored-By yok**, kod yorumu İngilizce). Push/merge yok; sormadan yapılmaz.
7. Oturuma sığmıyorsa: kapıdan geçen kısmı commit et, kalanı için yeni görev satırı + dosyası aç, bu görevi `bölündü` yap.

Bağlamı korumak için: büyük dosyaları bütün okuma (`AGENTS.md` "Oturum ve token düzeni"). Canlı oyun yalnız görevin **Kabul**ü isterse, `python3 tools/voxel.py shot` ile.

## GUI oracle tarifi (G)

GUI görevleri bu beş adımı izler; görev dosyaları "G" diye anar.

1. Java: `tests/behavior/java/GuiOracle.java` — operation yöntemi ekle (örnek `pauseScreen`, `grep -n "pauseScreen"`), `switch` koluna bağla (~485–496). Gerçek Java ekranı Unsafe/yansıma sahtelerle kurulur (`screen(...)`, `set/get` yardımcıları üstte).
2. `tools/behavior/cases.py` — `gui_<ad>_cases()` üreticisi (örnek `gui_pause_screen_cases`); `generators["gui"]` zincirine ekle (`grep -n 'generators\["gui"\]'`). T02'den sonra yer `cases_gui.py`'dir.
3. Rust: `crates/voxel/examples/behavior_adapter/gui.rs` içindeki `gui_result` match kolu (örnek `"pause_screen"`). Rust ekranı üretimdeki tipin kendisidir; adaptör yalnız kurar ve `draw` çağırır.
4. `docs/gui-oracle.md` kapsam tablosuna operation satırı.
5. `python3 tools/voxel.py parity --capture --family gui` yeşil olana dek yinele. Fark çıkarsa Rust'ı düzelt; beklenen değeri Rust çıktısından türetme.

Rust ekran kalıbı: `voxel-render/src/gui/{pause,death,disconnected}.rs` (ızgara: `layout.rs`, düğme/simge: `widgets.rs`, sprite: `atlas.rs`); oyuna bağlama `crates/voxel/src/{controls,gui,play}.rs` (`open_pause`, `receive_death` örnek).

## Görevler

Boyut: S ≈ yarım oturum, M ≈ bir oturum, L ≈ zor oturum (sığmazsa böl).

| Kimlik | Görev | Boy | Bağımlılık | Durum |
| --- | --- | --- | --- | --- |
| [T01](T01-adapter-gui-modulu.md) | `behavior_adapter` GUI modülüne böl | S | – | bitti |
| [T02](T02-cases-gui-modulu.md) | `cases.py` GUI üreticilerini ayır | S | T01 | bitti |
| [T03](T03-ekran-yigini-confirm.md) | Ekran yığını + `ConfirmScreen` | M | – | bitti |
| [T04](T04-options-durumu.md) | Seçenek değerleri ve `options.txt` | M | – | bitti |
| [T05](T05-slider-cycle-widget.md) | Kaydırıcı ve döngü düğmesi widget'ları | M | T04 | bitti |
| [T06](T06-options-ekrani.md) | `OptionsScreen` | M | T03, T05 | bitti |
| [T07](T07-video-ayarlari.md) | `OptionsList` + `VideoSettingsScreen` (FOV, GUI ölçeği) | M | T06 | bitti |
| [T08](T08-sohbet-ayarlari.md) | `ChatOptionsScreen` + sohbet ayarları çizime | M | T06 | bitti |
| [T09](T09-fare-ayarlari.md) | `MouseSettingsScreen` + duyarlılık | S | T06 | bitti |
| [T10](T10-dil-secimi.md) | `LanguageSelectScreen` + dil yeniden yükleme | L | T06 | bitti |
| [T10b](T10b-sohbet-dil-yenileme.md) | Dil değişince sohbet geçmişini yeniden çöz | S | T10 | bitti |
| [T11](T11-ipucu-ok-odagi.md) | Widget ipuçları + ok tuşlarıyla odak | M | T05 | bitti |
| [T11b](T11b-liste-klavye-odagi.md) | Ayar listelerinde klavye odağı (Tab, ok) | M | T11 | bitti |
| [T12](T12-click-hover-modeli.md) | `ClickEvent`/`HoverEvent` stil modeli + sohbet ipucu | M | – | bitti |
| [T13](T13-tiklama-eylemleri.md) | Sohbet tıklama eylemleri + `ConfirmLinkScreen` | M | T03, T12 | bitti |
| [T14](T14-locator-bar.md) | `TrackedWaypoint` + `LocatorBar` | M | – | bitti |
| [T15](T15-sohbet-kalanlari.md) | Sohbet taslağı, seçici izin düzeyi | S | – | bitti |
| [T20](T20-entity-paketleri.md) | Entity paketleri canlı oturumda | L | – | bitti |
| [T21](T21-entity-tick.md) | Entity tick ve interpolasyon canlı döngüde | M | T20 | bitti |
| [T21b](T21b-mob-interpolasyonu.md) | Mob'ların (`LivingEntity`) konum/dönüş interpolasyonu | M | T21 | bitti |
| [T22](T22-uzak-oyuncu.md) | Uzaktaki oyuncu çizimi | L | T21 | bitti |
| [T22b](T22b-ad-etiketi.md) | Uzaktaki oyuncunun ad etiketi | M | T22 | bitti (402a57a1) |
| [T23](T23-model-katmani-tablosu.md) | Entity model katmanı tablosu (oracle'dan) | L | – | bitti |
| [T24](T24-entity-cizim-iskeleti.md) | Entity çizim iskeleti + ilk mob | L | T22, T23 | bitti |
| [T24b](T24b-mob-cesitleri.md) | Pig/cow çeşitleri ve entity metadata okuyucusu | M | T24 | bitti (8f0774ef) |
| [T25](T25-yerdeki-esya.md) | Yerdeki eşya | M | T24 | bitti |
| [T25b](T25b-xp-kuresi.md) | XP küresi | S | T25 | bitti (0a0fd994) |
| [T25c](T25c-esya-fizigi.md) | Yerdeki eşyanın istemci fiziği (düşme, sekme) | M | T25 | bitti |
| [T25d](T25d-esya-akiskan.md) | Yerdeki eşya akışkanda (kaldırma, akıntı) | M | T25c | bitti (ae397885) |
| [T26](T26-entity-hedefleme.md) | Entity hedefleme, `Attack`/`Interact` | M | T21 | bitti (e9dae9f8) |
| [T26b](T26b-secici-ve-pick-esya.md) | Bakılan varlık UUID önerisi + `PickItemFromEntity` | S | T26 | bitti (932f7c82) |
| [T27](T27-hasar-olum-geri-bildirimi.md) | Mob hasar kırmızısı ve ölüm yatışı (`EntityEvent`/`HurtAnimation`/`DamageEvent`, `hurtTime`/`deathTime`); kaçış süresi gözlemi | M | T26, T24 | bitti (1985d1ae) |
| [T28](T28-insansi-zombi.md) | Zombi ailesi (`HumanoidModel` + zombi kolları) | L | T24, T27 | bitti (33ec5128) |
| [T29](T29-iskelet-piglin.md) | İskelet ve piglin aileleri (yay/arbalet/kılıç pozları) | M | T28 | bitti (05d7dc65) |
| [T30](T30-ciftlik-hayvanlari-2.md) | Çiftlik hayvanları 2: koyun, mantar ineği, tavuk, keçi, tavşan | M | T24 | bölündü (b56f4b9c) |
| [T30b](T30b-keci-tavsan-mantar-inegi.md) | Keçi ve mantar ineği | M | T30, T34d | bitti (9bb2c314) |
| [T30c](T30c-tavsan.md) | Tavşan (zıplama ve baş eğme anahtar kareleri) | M | T43b | bitti (ed3f293b) |
| [T31](T31-at-deve-ailesi.md) | At, eşek, katır, deve ve lama aileleri (T43b hazır; sığmazsa at ailesi / deve+lama diye böl) | L | T24 | açık |
| [T32](T32-evcil-yirtici-dortayak.md) | Evcil ve yırtıcı dört ayaklılar: kurt, kedi, vaşak, tilki, panda, kutup ayısı, hoglin, ravager, strider | L | T24 | açık |
| [T33](T33-eklembacaklilar-creeper.md) | Örümcek, böcek, arı ve creeper | M | T24 | bölündü (584d5eca) |
| [T33b](T33b-goz-sarj-bocek-ari.md) | Gümüş balığı, endermite, arı | M | T33 | bitti (3fb0a4f9) |
| [T33c](T33c-emissive-goz-katmani.md) | Emissive göz katmanı (örümcek, enderman) | S | T33, T34c | bitti (949699a3) |
| [T33d](T33d-enerji-girdabi.md) | Enerji girdabı ve şarjlı creeper | M | T33c | açık |
| [T34](T34-koylu-illager-golem.md) | Köylü, tüccar, illager, cadı, golemler ve enderman | L | T24 | bölündü (bc0ab848) |
| [T34b](T34b-illager-cadi-golem-enderman.md) | Illager ve cadı | M | T34 | bitti (48e17e52) |
| [T34c](T34c-golem-enderman-zombi-koylu.md) | Demir ve kar golemi, enderman, zombi köylü | L | T34b | bitti (f2c973a6) |
| [T34d](T34d-varlik-ustunde-blok.md) | Varlık üstünde blok durumu çizimi (balkabağı, çiçek, taşınan blok) | M | T34c | bitti (02ea54b4) |
| [T34e](T34e-bakir-golem.md) | Bakır golem | M | T43b | açık |
| [T35](T35-su-canlilari.md) | Su canlıları: kalamar, balıklar, yunus, aksolotl, kaplumbağa, kurbağa, nautilus, gardiyan | L | T24, T43b | açık |
| [T36](T36-ucanlar.md) | Uçanlar ve hayaletler: yarasa, papağan, allay, vex, phantom, blaze, ghast, breeze | M | T24 | açık |
| [T37](T37-boss-ve-ozel.md) | Boss ve özel mobların çizimi: wither, ender ejderhası, warden, shulker, creaking, balçıklar, armadillo, sniffer | L | T43b, T36 | açık |
| [T38](T38-araclar.md) | Araçlar: tekneler ve minecart ailesi | M | T24 | açık |
| [T39](T39-mermiler.md) | Mermiler ve fırlatılanlar | M | T24 | açık |
| [T40](T40-dekor-nesneler.md) | Dekor ve nesneler: zırh askısı, çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler | L | T24 | bölündü (bde91dfc) |
| [T40b](T40b-cerceve-tablo-dekor.md) | Çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler | L | T40 | açık |
| [T41](T41-ekipman-katmanlari.md) | Ekipman katmanları: `SetEquipment`, elde eşya, zırh, kafa eşyası, pelerin/elytra | L | T28, T22 | bölündü (d20fb045) |
| [T41b](T41b-zirh-katmani.md) | Zırh katmanı (`HumanoidArmorLayer`, boya, süs, parıltı) | L | T41 | bitti (c2bf0545) |
| [T41e](T41e-zirh-susu-parilti.md) | Zırh süsü, büyü parıltısı ve bileşen yaması | M | T41b | açık |
| [T41c](T41c-kafa-pelerin-elytra.md) | Kafa eşyası, pelerin, elytra, dürbün | M | T41 | bölündü (kafa eşyası: 4c7e5a5a) |
| [T41f](T41f-elytra-kafatasi.md) | Elytra, kafatası modelleri, gözde dürbün | M | T41c | açık |
| [T41g](T41g-koylu-cadi-esya.md) | Köylü, tüccar ve cadının tuttuğu ve taktığı eşya | S | T34b, T41c | bitti (2b8c193d) |
| [T41d](T41d-kullanim-pozlari.md) | Eşya kullanım pozları ve silah kol pozları | M | T41 | açık |
| [T42](T42-entity-golgesi.md) | Entity gölgesi (`submitShadow`) | M | T24 | bitti (d2c2f477) |
| [T43](T43-keyframe-animasyon-ve-animate.md) | `KeyframeAnimation` oynatıcısı, `Animate` paketi ve `EntityEvent` çizim durumları | L | T24 | bölündü (66840389) |
| [T43b](T43b-keyframe-oynatici.md) | `KeyframeAnimation` oynatıcısı ve `AnimationState` (ilk tüketici tavşan, T30c) | M | T43 | bitti (ed3f293b) |
| [T44](T44-binek-surusu.md) | Binek sürüşü: `PlayerInput`, `PlayerCommand`, `MoveVehicle`, `PaddleBoat` | L | T38, T31 | açık |
| [T45](T45-saldiri-gucu-ve-combat.md) | Saldırı gücü göstergesi ve `PlayerCombat*` paketleri | S | T26 | açık |
| PLAN-M4 | Kalan entity içeriğini görevlere böl (insansı/dört ayaklı aileler, zırh, ad etiketi, hasar/ölüm (T27 ayrıldı: kırmızı örtü ve ölüm yatışı), binekler, mermiler, çerçeve/tablo) | S | T24 | bitti (8ab67211) |
| PLAN-M8 | Blok entity çizicileri + `LevelEvent` + partikül sistemini görevlere böl (`TakeItemEntity` paketi + `ItemPickupParticle` eşya toplama animasyonu da burada: parçacık motoru olmadan yapılmaz) | S | T24 | açık |
| [T46](T46-mob-oznitelikleri.md) | Mob öznitelikleri (`UpdateAttributes`): can üstü, ölçek | S | T34c | bitti (99446dc2) |
| [PLAN-M3d](PLAN-M3d.md) | Kalan kap ekranları ve tarif kitabını görevlere böl | S | – | açık |
| [PLAN-M6](PLAN-M3d.md) | Ana menü, sunucu listesi, ilerlemeler, kitap/tabela ve kalan ekranları görevlere böl | S | – | açık |
| PLAN-M7 | Ses: bağımlılık kararı ([karar](../audio-backend-decision.md): çıkış `cpal`, çözücü `lewton`) ve görevlere bölme | S | – | bitti |
| [T60](T60-sounds-json-secim.md) | `sounds.json` çözümü ve ağırlıklı ses seçimi | M | – | açık |
| [T61](T61-ses-paketleri-kanal.md) | `Sound`/`SoundEntity`/`StopSound` paketleri ve `SoundEngine` kanal parametreleri | M | T60 | açık |
| [T62](T62-ogg-vorbis-cozumu.md) | Ogg/Vorbis çözümü (`lewton`, JOrbis ile PCM karşılaştırması) | M | – | açık |
| [T63](T63-ses-cikisi-cpal.md) | Ses çıkışı (`cpal`): kanal havuzu, sönüm, perde, miks | L | T61, T62 | açık |
| [T64](T64-ses-tetikleri-muzik.md) | Oyun ses tetikleri (adım, kırma, hasar, ortam) ve müzik | L | T63 | açık |
| [T65](T65-ses-ayarlari-ekrani.md) | Ses ayarları ekranı | M | T63, T06 | açık |
| PLAN-M9 | Çevrimiçi kimlik, yeniden yapılandırma, kaynak paketi görevlere böl | S | – | açık |

Görev yazarken şablon: Amaç · Bağımlılık · Oku · Yapılacaklar · Oracle · Kabul · Kapsam dışı · Belgeler · Devir notu (≤ 60 satır).
