# P01.b — Vec2 distanceToSqr sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.distanceToSqr(Vec2)` metodunun iki f32 bileşen farkını kareleyip toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:72-76` içindeki `distanceToSqr` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `p.x - this.x` ve `p.y - this.y` farkları f32 olarak ayrı hesaplanır.
- Önce x farkının, sonra y farkının karesi alınır; iki kare kaynak sırasıyla f32 olarak toplanır.
- Signed zero, ara yuvarlama, taşma, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.
- Sonuç tek f32 değeridir; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Sonlu iki Vec2 | Kareli uzaklık |
| `order-and-rounding` | Büyük x farkı ve küçük y farkı | f32 toplama yuvarlaması |
| `signed-zero` | İşaretli sıfır bileşenleri | Kare ve toplam bitleri |
| `overflow` | En büyük sonlu f32 bileşenleri | Infinity bitleri |
| `special-values` | Infinity ve NaN bileşenleri | Özel f32 sonucu |

Girdiler `family: "math"`, `operation: "vec2_distance_to_sqr"`, `first` ve `second` Vec2 nesnelerini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec2::distance_to_sqr`, Java'nın çıkarma, kare alma ve kaynak sıralı toplama adımlarını uygular. `examples/behavior_adapter.rs` operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `distanceToSqr` çağrısıyla bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec2.distanceToSqr(Vec2)` metodunu karşılayan `Vec2::distance_to_sqr` metodunu içerir. Java oracle doğrudan `distanceToSqr` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda f32 çıkarma, kare alma, kaynak sıralı toplama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve distanceToSqr sonuçlarının f32 bit değerleriyle sınırlıdır. Vec2 add, dot, rotate ve diğer yardımcıları ayrı davranış dilimleridir.
