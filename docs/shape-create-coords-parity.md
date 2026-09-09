# P01.d — Shapes.create(double...) doğrulaması

`src/shape.rs`, Java `Shapes.create(double...)` overload'ını `Shapes::create_from_coords` ile uygular. `examples/behavior_adapter.rs`, `kind: "create_coords"` girdisini ham f64 endpoint'leriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 710 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **710/710** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1164/1164** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_coords` senaryosunda cube, ters endpoint empty sonucu ve ArrayVoxelShape fallback'i karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80`.

Birleşik referans kimliği: `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80`.

Yerel raporlar: `compare-ioen6ccd` (math debug), `compare-r587dixs` (math release), `compare-s2nusmi5` (birleşik debug) ve `compare-2zqsf1m9` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik ham koordinat girdisiyle sınırlıdır. `findBits` yardımcı metodunun package-private doğrudan erişimi, özel NaN payload'ları ve shape üretimindeki diğer merger dalları ayrı kapsamdır.
