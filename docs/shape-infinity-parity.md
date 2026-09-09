# P01.d — Shapes.INFINITY doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 650 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **650/650** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1104/1104** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Birleşik referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Yerel raporlar: `compare-krjmldv0` (math debug), `compare-ufbmlufj` (math release), `compare-o61mnrf6` (birleşik debug) ve `compare-6fi73fsy` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
