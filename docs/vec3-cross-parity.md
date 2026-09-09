# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 973 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **973/973** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1475/1475** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1475 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5b9ac04dfbc4d0f113189cc9a302a301459740911a59e19bb68a172015f61780`, birleşik referans `6be1cb26871cc94c222742a0c5ccb50c6ef5df50a6b4f33206aeefdd805d2a60` kimliğindedir.

Yerel raporlar: `compare-yzj8g47` (math debug), `compare-0gyul_vz` (math release), `compare-1qxbdgol` (birleşik debug), `compare-9okhsl5l` (birleşik release) ve `verify-v5w4qfr` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
