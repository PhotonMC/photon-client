# P01.b — Vec2 negated doğrulaması

`src/math.rs`, Java `Vec2.negated()` metodunu karşılayan `Vec2::negated` metodunu içerir. Java oracle doğrudan `negated` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1398 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1398/1398** math senaryosunda eşleşti.
- Beş yeni senaryoda unary f32 negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referansı `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`, birleşik referans `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliklerindedir.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release) ve `verify-5ju3ok97` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve negated sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
