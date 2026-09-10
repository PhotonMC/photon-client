# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1450 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1450/1450** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Math referansı `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`, birleşik referans `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11` kimliğindedir.

Yerel raporlar: `compare-qtqxwt9b` (math debug), `compare-ebava42g` (math release), `compare-x0hkkvkv` (birleşik debug), `compare-x5wgwswa` (birleşik release) ve `verify-rc21vyy9` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
