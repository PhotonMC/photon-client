# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 710 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **710/710** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1164/1164** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1164 senaryoluk Java referansını yeniden doğruladı.

Math referansı `413d8249156e0599ab7e9f1c30d370138a709fdb956352fff6e66dc1d593ffa9`, birleşik referans `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80` kimliğindedir.

Yerel raporlar: `compare-ioen6ccd` (math debug), `compare-r587dixs` (math release), `compare-s2nusmi5` (birleşik debug), `compare-2zqsf1m9` (birleşik release) ve `verify-tm97vdnp` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
