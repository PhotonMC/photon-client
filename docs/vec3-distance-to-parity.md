# P01.b — Vec3 distanceTo doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 710 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **710/710** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1164/1164** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1164 senaryoluk Java referansını yeniden doğruladı.

Math referansı `413d8249156e0599ab7e9f1c30d370138a709fdb956352fff6e66dc1d593ffa9`, birleşik referans `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80` kimliğindedir.

Yerel raporlar: `compare-ioen6ccd` (math debug), `compare-r587dixs` (math release), `compare-s2nusmi5` (birleşik debug), `compare-2zqsf1m9` (birleşik release) ve `verify-tm97vdnp` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
