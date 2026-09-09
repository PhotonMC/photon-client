# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 660 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **660/660** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1114/1114** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8`.

Birleşik referans kimliği: `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8`.

Yerel raporlar: `compare-3kg8m9ta` (math debug), `compare-6crwfl2x` (math release), `compare-_35t8r3l` (birleşik debug) ve `compare-g5oda9d_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
