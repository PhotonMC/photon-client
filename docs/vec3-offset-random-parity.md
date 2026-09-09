# P01.b — Vec3 offsetRandom doğrulaması

`src/math.rs`, Java `Vec3.offsetRandom(RandomSource, float)` ve `Vec3.offsetRandomXZ(RandomSource, float)` overload'larını karşılar. Rust adaptörü aynı seed ile iki Rust `RandomSource` örneği kurar; Java oracle iki overload'u doğrudan çağırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 859 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **859/859** math senaryosunda eşleşti.
- Beş yeni senaryoda üç ve iki `nextFloat()` çekişi, f32 offset ara hesabı, negatif offset, signed zero, subnormal offset ve özel Vec3 değerleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1313/1313** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1313 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b65c658bf8c61413f639524850d054786ff936198bd6d0bb52662da232a61d77`, birleşik referans `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2` kimliğindedir.

Yerel raporlar: `compare-_2ksshx8` (math debug), `compare-z3kwikqe` (math release), `compare-6n7i4k79` (birleşik debug), `compare-se85bnok` (birleşik release) ve `verify-1_zhkgp3` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde iki random offset overload'unun Vec3 snapshot'larıyla sınırlıdır. `RandomSource`'un diğer üreticileri ve `Vec3`'ün kalan yardımcıları ayrı dilimlerdir.
