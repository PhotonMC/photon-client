# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1249 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1249/1249** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1751 senaryoluk Java referansını yeniden doğruladı.

Math referansı `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`, birleşik referans `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0` kimliğindedir.

Yerel raporlar: `compare-dj1scyn3` (math debug), `compare-1g72i2us` (math release), `compare-f0bjlm2c` (birleşik debug), `compare-eaqfihsj` (birleşik release) ve `verify-p2v9lu2o` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
