# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 680 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **680/680** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1134/1134** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1134 senaryoluk Java referansını yeniden doğruladı.

Math referansı `3e58e449e5df13343950e698291673b0ac6cbfd83317a41d1c5806c3eab7f0f1`, birleşik referans `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7` kimliğindedir.

Yerel raporlar: `compare-yr90fi_v` (math debug), `compare-a6r_92in` (math release), `compare-aiddld1f` (birleşik debug), `compare-micscpjd` (birleşik release) ve `verify-z6172w56` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
