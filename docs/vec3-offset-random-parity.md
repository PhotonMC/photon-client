# P01.b — Vec3 offsetRandom doğrulaması

`src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 849 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **849/849** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1303/1303** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1303 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e29959e1e804ec49cc4359adf306d0af3b2268e8c97cc1a9a11aeb3872e657a2`, birleşik referans `2934949cd24cd9684ec3034e431974ec99773d6473c977dd6771ec28e892c647` kimliğindedir.

Yerel raporlar: `compare-huwe3iyu` (math debug), `compare-mentlii0` (math release), `compare-fw1d9dde` (birleşik debug), `compare-lx5vlkq_` (birleşik release) ve `verify-mw2w2ovh` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
