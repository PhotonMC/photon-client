# P01.d — Shapes.INFINITY doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 627 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **627/627** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1081/1081** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Birleşik referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Yerel raporlar: `compare-tg0rl658` (math debug), `compare-713odvz2` (math release), `compare-pnvz2myv` (birleşik debug) ve `compare-_bmqdjjb` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
