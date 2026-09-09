# P01.b — Vec3 dot doğrulaması

`src/math.rs`, Java `Vec3.dot(Vec3)` metodunu karşılayan `Vec3::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 720 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **720/720** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1174/1174** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1174 senaryoluk Java referansını yeniden doğruladı.

Math referansı `822cbae2fbb27092a3cb8dc7d82d032e30487876d5f2d0431f6215abf05038f4`, birleşik referans `4bff41b67362ba1b385c9a64628e8b701801206248af7733a736d3304b96f5c0` kimliğindedir.

Yerel raporlar: `compare-__o6jo0i` (math debug), `compare-tkeflqna` (math release), `compare-8yiueogs` (birleşik debug), `compare-btgxa_ac` (birleşik release) ve `verify-754kov0_` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 çifti ve dot sonuçlarının f64 bit snapshot'larıyla sınırlıdır. `cross`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
