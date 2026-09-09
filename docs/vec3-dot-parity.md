# P01.b — Vec3 dot doğrulaması

`src/math.rs`, Java `Vec3.dot(Vec3)` metodunu karşılayan `Vec3::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 917 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **917/917** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1419/1419** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1419 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7c835b893c95d986797a072a7b69eba5f1f31dad85d36d5bd51ba6394a221696`, birleşik referans `cfb0f4a3444412c75186c0d8b209a59632b9d62debcc435aa548bf4c364d8656` kimliğindedir.

Yerel raporlar: `compare-dvzelshx` (math debug), `compare-sy72qqem` (math release), `compare-ng1mczig` (birleşik debug), `compare-7brjjbpz` (birleşik release) ve `verify-3oltsdwp` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 çifti ve dot sonuçlarının f64 bit snapshot'larıyla sınırlıdır. `cross`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
