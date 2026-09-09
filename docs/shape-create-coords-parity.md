# P01.d — Shapes.create(double...) doğrulaması

`src/shape.rs`, Java `Shapes.create(double...)` overload'ını `Shapes::create_from_coords` ile uygular. `examples/behavior_adapter.rs`, `kind: "create_coords"` girdisini ham f64 endpoint'leriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 675 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **675/675** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1129/1129** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_coords` senaryosunda cube, ters endpoint empty sonucu ve ArrayVoxelShape fallback'i karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cde30e979941342736ed7dcc8995baa3a0f857ed47dbf23c18cad23127043901`.

Birleşik referans kimliği: `cde30e979941342736ed7dcc8995baa3a0f857ed47dbf23c18cad23127043901`.

Yerel raporlar: `compare-d110h2ng` (math debug), `compare-t3jgwbff` (math release), `compare-lkmx56jk` (birleşik debug) ve `compare-666jyr9x` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik ham koordinat girdisiyle sınırlıdır. `findBits` yardımcı metodunun package-private doğrudan erişimi, özel NaN payload'ları ve shape üretimindeki diğer merger dalları ayrı kapsamdır.
