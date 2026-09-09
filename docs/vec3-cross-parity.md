# P01.b — Vec3 cross doğrulaması

`src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 777 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **777/777** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1231/1231** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1231 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d6723874f1f8073346a0f729f2c06b2e00e2dcd34baad1a33733698d293350d3`, birleşik referans `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe` kimliğindedir.

Yerel raporlar: `compare-da6k05zu` (math debug), `compare-jem8yqw3` (math release), `compare-r10lr0oe` (birleşik debug), `compare-rb0k9jx5` (birleşik release) ve `verify-04cazxan` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
