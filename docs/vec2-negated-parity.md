# P01.b — Vec2 negated doğrulaması

`src/math.rs`, Java `Vec2.negated()` metodunu karşılayan `Vec2::negated` metodunu içerir. Java oracle doğrudan `negated` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 983 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **983/983** math senaryosunda eşleşti.
- Beş yeni senaryoda unary f32 negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1485 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420532c31a46dc73c7456731fb356a539ca36511c8ae3b916c7b882d3703a8e5`, birleşik referans `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671` kimliklerindedir.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug), `compare-09rh6rbh` (birleşik release) ve `verify-dcqd08ai` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve negated sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
