# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1301 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1301/1301** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Birleşik referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Yerel raporlar: `compare-zq98bs3k` (math debug), `compare-zgg863t0` (math release), `compare-yz31bmcf` (birleşik debug) ve `compare-90dzj4qp` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
