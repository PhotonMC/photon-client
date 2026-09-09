# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1052 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1052/1052** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1554/1554** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1554 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d4f17c1024ea8a511a4611971e222dcdaa1a00ec6c1baa442420a0f051c914cc`, birleşik referans `f615eef08df36fc7705e9f9da886cfc498964867c630a79cb4bf7f15f3c0bae1` kimliğindedir.

Yerel raporlar: `compare-xj4m01k2` (math debug), `compare-ix4n2052` (math release), `compare-e0lm9mqh` (birleşik debug), `compare-pp8moc03` (birleşik release) ve `verify-r4qtgn3b` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
