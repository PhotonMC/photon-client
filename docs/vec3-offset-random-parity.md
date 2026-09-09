# P01.b — Vec3 offsetRandom doğrulaması

`src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 813 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **813/813** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1267/1267** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1267 senaryoluk Java referansını yeniden doğruladı.

Math referansı `28f7cb5106f4afaea9ccbb7f1a5601e99e2bf46d20a240f2d4288eb8fa0810ca`, birleşik referans `6b1e215c94a31ad5e80b7ab921054436873666bf0c583fb7b768cc855632bde2` kimliğindedir.

Yerel raporlar: `compare-nv2dy1tn` (math debug), `compare-zi0twrtb` (math release), `compare-lny6p0bp` (birleşik debug), `compare-g4_lvtxx` (birleşik release) ve `verify-nf8ea9dc` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
