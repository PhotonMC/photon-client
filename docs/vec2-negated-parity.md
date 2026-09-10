# P01.b — Vec2 negated doğrulaması

`src/math.rs`, Java `Vec2.negated()` metodunu karşılayan `Vec2::negated` metodunu içerir. Java oracle doğrudan `negated` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2166 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2166/2166** math senaryosunda eşleşti.
- Beş yeni senaryoda unary f32 negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2668/2668** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2668 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve negated sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
