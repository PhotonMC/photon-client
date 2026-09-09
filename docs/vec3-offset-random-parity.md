# P01.b — Vec3 offsetRandom doğrulaması

`src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1113 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1113/1113** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1615 senaryoluk Java referansını yeniden doğruladı.

Math referansı `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`, birleşik referans `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliğindedir.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug), `compare-1jjs0aun` (birleşik release) ve `verify-71u5jh9x` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
