# P01.b — Vec2 negated sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.negated()` metodunun her f32 bileşenini unary negation ile tersleyip yeni Vec2 oluşturmasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:78-80` içindeki `negated` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç yeni bir Vec2'dür; x ve y bileşenleri kaynak sırayla unary f32 negation'dan geçirilir.
- Negation yalnızca sign bitini tersler; sonlu değerlerin büyüklüğü, subnormal bitleri ve Infinity korunur.
- Signed zero işaret değiştirir.
- NaN payload bitleri korunurken NaN sign bit'i terslenir; sonuçlar ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu pozitif ve negatif bileşenler | İşareti çevrilmiş Vec2 |
| `signed-zero` | Negatif ve pozitif sıfır | Çevrilmiş sıfır işaretleri |
| `subnormal` | Pozitif ve negatif en küçük subnormal | Payload korunarak çevrilmiş bitler |
| `infinity-and-boundary` | En büyük sonlu değer ve negatif Infinity | Negatif sınır ve pozitif Infinity |
| `nan-payload` | Farklı sign/payload NaN değerleri | Ters sign, korunmuş payload |

Girdiler `family: "math"`, `operation: "vec2_negated"` ve `first` Vec2 nesnesini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında Vec2 snapshot'ı olarak döner.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec2::negated`, Java'nın iki unary f32 negation adımını uygular. `examples/behavior_adapter.rs` operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `negated` çağrısıyla bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec2.negated()` metodunu karşılayan `Vec2::negated` metodunu içerir. Java oracle doğrudan `negated` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda unary f32 negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve negated sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
