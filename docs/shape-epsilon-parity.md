# P01.d — Shapes.EPSILON ve BIG_EPSILON doğrulaması

`src/shape.rs`, Java `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitlerini associated constant olarak sunar. `examples/behavior_adapter.rs`, bu değerleri doğrudan Rust API'sinden f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 765 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **765/765** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1219/1219** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `constants` senaryosunda iki public sabitin ham f64 bitleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a`.

Birleşik referans kimliği: `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a`.

Yerel raporlar: `compare-ya8jvd1o` (math debug), `compare-rh0c0wxr` (math release), `compare-u_e3yyct` (birleşik debug) ve `compare-326ehkgq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt public sabit değerleriyle sınırlıdır. Bu eşiklerin kullanıldığı package-private merger yardımcılarının ayrı davranış dalları sonraki kapsamdır.
