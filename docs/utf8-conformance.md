# P03.b — UTF-8 string referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **141 Java davranış testi hazırdır; Rust implementasyonu henüz yoktur**. Önceki 346 math/timer/wire senaryosu değişmedi. UUID, konum, bitset ve diğer P03.b biçimleri bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `net/minecraft/network/Utf8String.java` içindeki `read` ve `write` incelendi. `FriendlyByteBuf.readUtf/writeUtf` ve `codec/ByteBufCodecs.stringUtf8` bu işlemlere yönlendiriyor; varsayılan string sınırı 32767. Adaptör doğrudan orijinal `Utf8String` sınıfını çağırır; üst katman sınıflarının başlatılması veya gerçek ağ bağlantısı bu testte yapılmaz.

Netty buffer/common yanında `netty-codec-base-4.2.15.Final.jar` gerekir. Yerel JAR'ın SHA-1 değeri `tests/behavior/reference.json` içine sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi. Java/Netty algoritmaları adaptörde yeniden uygulanmadı.

## Davranış sözleşmesi

- Uzunluk sınırı Java UTF-16 kod birimleri cinsindendir. Unicode scalar sayısı, ekranda görünen karakter sayısı veya UTF-8 bayt sayısı aynı ölçü değildir. Supplementary karakterler iki birim kullanır; birleşen karakterler birleştirilmez.
- Okumada VarInt uzunluk başlığından sonra kodlanmış bayt sınırı, negatif uzunluk ve kullanılabilir veri denetlenir. İzin verilen kodlanmış uzunluk bu dilimdeki sınırlar için `3 × max_length` olur. Başlık hataları önceki VarInt tüketim davranışını korur.
- Gövde çözüldükten sonra UTF-16 uzunluğu ayrıca denetlenir. Bu son denetim başarısız olduğunda gövde tüketilmiş olur; gövde eksikliği veya kodlanmış uzunluk hatasında gövde tüketilmez. Kalan baytlar karşılaştırmanın parçasıdır.
- Bozuk UTF-8 dizileri her zaman hata üretmez. Referans decoder'ın replacement karakterlerini nasıl grupladığı ölçülür; genel bir Rust lossy dönüşümünün aynı olduğu varsayılmaz.
- Yazmada UTF-16 uzunluğu denetlenir, ardından Netty ile kodlanmış uzunluk başlığı ve gövde eklenir. Mevcut yazma sınırı hatalarında hedef buffer'a yeni veri eklenmez; önceden bulunan baytlar korunur.
- Java string'i eşleşmemiş surrogate birimleri taşıyabilir. Bunların Netty ile yazılması standart Unicode scalar kodlamasına indirgenemez: eşleşmemiş yüksek surrogate'ten sonraki birim de tüketilebilir ve sonuç o birime bağlıdır. İleride Rust API'sinde yalnızca `&str` kabul etmek bu girdileri temsil edemez; UTF-16 birim girdisi korunmalıdır.

İlk corpus `max_length` değerlerini 0–32767 aralığında kullanır. Negatif/aşırı büyük limitler, limit çarpımı taşması, bellek yetersizliği, farklı `CharSequence` implementasyonları ve tüm bozuk Unicode dizileri doğrulanmış değildir. Heap buffer ve sıfırdan farklı reader index kullanıldı; direct/composite buffer varyantları ve çoklu string akışları kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Yazma | 50 | Boş/NUL/ASCII, Türkçe, birleşen karakterler, UTF genişlik sınırları, supplementary ve eşleşmemiş surrogate dizileri; birim sınırının altı/eşiti/üstü; 127/128 bayt ve 32767 varsayılan sınırı |
| Okuma | 91 | Bağımsız bayt dizileri; geçerli metin, bozuk/overlong/truncated UTF-8, surrogate ve Unicode aralığı dışındaki diziler; eksik/fazla uzun/noncanonical VarInt, gövde sınırları, kalan veri |

`family: "utf8"`, `operation: "read" | "write"`, `max_length` JSON integer ve `prefix_hex` küçük harfli bayt hex string ortak girdilerdir. `read` için `hex` okunacak başlık/gövdeyi taşır. `write` için `utf16_hex`, her kod birimi dört hex hane olacak şekilde Java string'ini taşır; JSON parser'ın surrogate dönüştürmesine güvenilmez.

Okuma adaptörü buffer başına `prefix_hex` koyup reader index'i bu alanın sonuna taşır. Başarılı sonuç `utf16_hex`, `utf16_length`, `outcome` içerir. Hata sonucu `outcome` ve `error` içerir. Her iki durumda `consumed` başlangıç reader index'ine göre tüketimi, `remaining_hex` kalan veriyi gösterir.

Yazma adaptörü mevcut `prefix_hex` üzerine ekler. Her sonuç `outcome`, eklenen bayt sayısı `written` ve prefix dahil bütün `hex` çıktısını içerir; hatada ayrıca `error` bulunur. Yazma çıktısı yeniden decode edilerek beklenen değer üretilmez; bağımsız okuma senaryoları ayrıdır.

Hata kategorileri `decoder_error`, `encoder_error`, `buffer_underflow`, `too_big`. İlk ikisi ilgili Netty exception grubudur; bütün uzunluk nedenlerini ayrı hata kodlarına bölmez. Tüketim ve buffer çıktısı da kesin karşılaştırılır. Java exception sınıfı yalnızca `diagnostics` içindedir; exception mesajı/stack trace eşitlik ölçütü değildir. Bu gözlemler birbirinden ayırt edilemeyen iç kontrol sıralarını kanıtlamaz.

## Çalıştırma ve kanıt

Depo kökünden, Java koşularını sırayla çalıştır:

```bash
python3 tools/behavior/run.py capture --family utf8
python3 tools/behavior/run.py verify --family utf8
```

141 senaryo iki ayrı JVM'de aynı sonucu verdi; kayıtlı referans iki yeni JVM ile yeniden doğrulandı. UTF-8 referans kimliği: `f8d898f024e107b2f8029166e17d4b16282945620c06c5a01a1c4739a43e5935`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Yeni aileyle toplam 487 Java senaryosu yakalandı ve iki yeni JVM çalıştırmasıyla kayıtla eşitliği doğrulandı. Birleşik referans kimliği: `9b7f4d3c9c59cddd1ed96162a13c8f308f8d38df91a69d445ec5f31bc0278799`. Dispatcher'a UTF-8 eklenmesi kaynak kimliğini değiştirdiği için önceki üç aile de yeni kimlikle açıkça yakalandı; eski 346 girdinin ve sonuçlarının önceki kayıtla aynı olduğu ayrıca kontrol edildi. Eski kayıtlar değiştirilmedi.

Mevcut Rust kodunu çalıştırmak için desteklenen aileleri açıkça seç:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family math --family timer --family wire
python3 tools/behavior/run.py compare --family math --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

Bu 346 senaryo debug/release profillerinde yeniden eşleşti. Varsayılan bütün-aile karşılaştırması UTF-8 henüz uygulanmadığından başarısız olur; yeni testler sessizce atlanmaz. Metin birimleri, uzunluk, tüketim, kalan/yazılan baytlar ve hata alanları üzerinde tek alan mutasyonlarının karşılaştırıcıda fark ürettiği ayrıca kontrol edildi.

Yerel raporlar `capture-k75g0ms0`, `verify-8ukg9f37`, `capture-8ib0wyre`, `verify-8n3e4i84`, `compare-aokf6gii`, `compare-xx9zff_o` ve `compare-82ziigmd` dizinlerinde, `.data/runs/behavior/` altında bulunur. `capture-k75g0ms0/contract-checks.json` eski kapsamın değişmediği ve mutasyon kontrollerinin kaydıdır. Eşzamanlı ilk birleşik denemedeki JVM performans dosyası uyarısı stdout'u bozduğu için koşu başarısız sayıldı; seri tekrar başarılı oldu. Uyarı satırlarını ayıklayarak test çıktısı kabul edilmedi.

Sonraki adım bu sabit referansa karşı UTF-8 Rust davranış dilimidir. Beklenen çıktılar Rust'a göre değiştirilmeyecek.


## Client-parity doğrulaması

`src/utf8.rs`, `Utf8String` okuma/yazma davranışını kapsar. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. `src/main.rs` hâlâ boştur. UUID, konum, bitset ve diğer P03 biçimleri bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `Utf8String` okuma/yazma yolları kaynak incelemesinde belirlendi. `Utf8Oracle.java` doğrudan orijinal sınıfı çağırır; senaryo ve beklenen çıktılar değiştirilmedi. Ayrıntılı sözleşme ve 141 senaryoluk kapsam [UTF-8 referans belgesindedir](#sözleşme).

| Rust API | Korunan davranış |
| --- | --- |
| `utf8_max_bytes` | Kodlanmış sınır `3 × max_length` olarak hesaplanır |
| `read_utf8(&mut &[u8], usize)` | VarInt başlığı, kodlanmış/çözülmüş uzunluk ve kullanılabilir veri denetimleri; hata durumunda referansın tüketim/kalan veri davranışı |
| `write_utf8(&mut Vec<u8>, &[u16], usize)` | UTF-16 birim sınırı, Netty uyumlu kodlama ve kodlanmış sınır; hatada hedefe ekleme yapılmaz |
| Dahili Java uyumlu çözümleyici | Geçerli diziler, bozuk/overlong/kesik dizilerde replacement gruplaması, surrogate ve aralık dışı diziler |
| Dahili Netty uyumlu kodlayıcı | Eşleşmemiş surrogate sonrası birimin ham bayt davranışı dahil lone ve eşleşmemiş surrogate eşlemesi |

Yazma girdisi UTF-16 birimleri olarak alınır; yalnızca `&str` kabul etmek eşleşmemiş surrogate girdilerini temsil edemez. Okuma çıktısı UTF-16 birimleridir; başarılı çözümleme eşleşmemiş surrogate içermez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 487 senaryoluk kayıtlı Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **487/487** eşleşti: 180 math + 22 timer + 144 wire + 141 utf8.
- Karşılaştırma metin birimleri, uzunluk, tüketim, kalan/yazılan baytlar ve hata kategorilerini kapsar; float toleransı yoktur.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `9b7f4d3c9c59cddd1ed96162a13c8f308f8d38df91a69d445ec5f31bc0278799`. UTF-8 ailesi kimliği: `f8d898f024e107b2f8029166e17d4b16282945620c06c5a01a1c4739a43e5935`.

Yerel raporlar: `verify-cwrm1vpb`, `compare-irkxihos` (debug 487), `compare-ynbbkpt5` (release 487) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir. Binary ailesi eklendiğinden mevcut Rust karşılaştırmasında `--family math --family timer --family wire --family utf8` seçilir.

P03.b kapsamında UUID ve konum için [43 Java referans testi](binary-conformance.md) hazır; sıradaki küçük dilim Rust implementasyonudur. Long array ve bitset henüz kapsanmadı.
