# P01.b — Vec2 negated doğrulaması

`src/math.rs`, Java `Vec2.negated()` metodunu karşılayan `Vec2::negated` metodunu içerir. Java oracle doğrudan `negated` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 893 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **893/893** math senaryosunda eşleşti.
- Beş yeni senaryoda unary f32 negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1375/1375** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1375 senaryoluk Java referansını yeniden doğruladı.

Math referansı `14e060de3e2dfd133faf305681288b099fffc157d99d2140efcced661186142a`, birleşik referans `7c9538198afb189201c05520d1d9beb9d8b8ed1ecf818a5751ba7c9cf2774c30` kimliklerindedir.

Yerel raporlar: `compare-gnxkpf0j` (math debug), `compare-u696_8k6` (math release), `compare-wdb79blv` (birleşik debug), `compare-_hlw5kud` (birleşik release) ve `verify-u0of387` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve negated sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
