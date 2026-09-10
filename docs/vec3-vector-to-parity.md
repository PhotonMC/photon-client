# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1398 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1398/1398** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referansı `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`, birleşik referans `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğindedir.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release) ve `verify-5ju3ok97` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
