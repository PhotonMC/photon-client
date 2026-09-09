# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 635 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **635/635** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1089/1089** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Birleşik referans kimliği: `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07`.

Yerel raporlar: `compare-ihodhcd4` (math debug), `compare-1euc1ro8` (math release), `compare-_c9tjida` (birleşik debug) ve `compare-jbm02l6z` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
