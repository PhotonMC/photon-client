# P01.d — Shapes.create(double...) doğrulaması

`src/shape.rs`, Java `Shapes.create(double...)` overload'ını `Shapes::create_from_coords` ile uygular. `examples/behavior_adapter.rs`, `kind: "create_coords"` girdisini ham f64 endpoint'leriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 923 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **923/923** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1425/1425** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_coords` senaryosunda cube, ters endpoint empty sonucu ve ArrayVoxelShape fallback'i karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422`.

Birleşik referans kimliği: `46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422`.

Yerel raporlar: `compare-85j8zrrj` (math debug), `compare-eq8g_l1t` (math release), `compare-ufi9ykko` (birleşik debug) ve `compare-fuqao9g1` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik ham koordinat girdisiyle sınırlıdır. `findBits` yardımcı metodunun package-private doğrudan erişimi, özel NaN payload'ları ve shape üretimindeki diğer merger dalları ayrı kapsamdır.
