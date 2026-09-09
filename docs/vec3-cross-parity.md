# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1040 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1040/1040** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1542/1542** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1542 senaryoluk Java referansını yeniden doğruladı.

Math referansı `a21163be0f5e47c3d99a4468d2f3b67c6d04d0dcee95ca6b70f49b581fac358c`, birleşik referans `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126` kimliğindedir.

Yerel raporlar: `compare-_rqqaqfb` (math debug), `compare-v1ltm6cc` (math release), `compare-1lz8q4ar` (birleşik debug), `compare-fvdcbn0r` (birleşik release) ve `verify-nud8oh4e` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
