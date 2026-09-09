# P01.b — Vec3 dot doğrulaması

`src/math.rs`, Java `Vec3.dot(Vec3)` metodunu karşılayan `Vec3::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 941 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **941/941** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1443/1443** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1443 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7aaf16bdfc85f793bd25e0d4d234f6988f30874d554dc65c32ce3d0ec4f85363`, birleşik referans `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6` kimliğindedir.

Yerel raporlar: `compare-9rd6f2k4` (math debug), `compare-azcslb8q` (math release), `compare-9epmwbb2` (birleşik debug), `compare-vdfamuam` (birleşik release) ve `verify-povrla2w` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 çifti ve dot sonuçlarının f64 bit snapshot'larıyla sınırlıdır. `cross`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
