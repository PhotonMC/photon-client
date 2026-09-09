# P01.b — Vec3 offsetRandom doğrulaması

`src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1092 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1092/1092** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1594/1594** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1594 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8475d3304542000c5493cb030a2dfb280baa6a64c19a4ba3eb09d75f9c0e67a8`, birleşik referans `a15082ba584c6d0bca4bc52d80fd5da8fc49f05193cec75afd71cf0af0c4ee0d` kimliğindedir.

Yerel raporlar: `compare-ta5w_83f` (math debug), `compare-vzkk24q9` (math release), `compare-271dl4fv` (birleşik debug), `compare-y8i46uni` (birleşik release) ve `verify-3t751q44` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
