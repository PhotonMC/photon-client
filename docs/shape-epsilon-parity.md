# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1354 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Yerel raporlar: `compare-z9ypl4_s` (math debug), `compare-0xhgd0yz` (math release), `compare-ote21q3t` (birleşik debug) ve `compare-r5nijsku` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
