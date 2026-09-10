# P01.d — Shapes.create(double...) doğrulaması

`src/shape.rs`, Java `Shapes.create(double...)` overload'ını `Shapes::create_from_coords` ile uygular. `examples/behavior_adapter.rs`, `kind: "create_coords"` girdisini ham f64 endpoint'leriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_coords` senaryosunda cube, ters endpoint empty sonucu ve ArrayVoxelShape fallback'i karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Yerel raporlar: `compare-hiau72j0` (math debug), `compare-bttpme0o` (math release), `compare-k17hah5m` (birleşik debug) ve `compare-hiiz6u35` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik ham koordinat girdisiyle sınırlıdır. `findBits` yardımcı metodunun package-private doğrudan erişimi, özel NaN payload'ları ve shape üretimindeki diğer merger dalları ayrı kapsamdır.
