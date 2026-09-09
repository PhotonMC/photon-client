# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 759 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **759/759** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1213/1213** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `dcbe4a62af107a8ef7ed2c1dd52518e6b9e2f063bfc9f0c0ba5565b0ba4617f8`.

Birleşik referans kimliği: `dcbe4a62af107a8ef7ed2c1dd52518e6b9e2f063bfc9f0c0ba5565b0ba4617f8`.

Yerel raporlar: `compare-iodxkzgj` (math debug), `compare-_orh4azt` (math release), `compare-xeqdw852` (birleşik debug) ve `compare-bk_l5btc` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
