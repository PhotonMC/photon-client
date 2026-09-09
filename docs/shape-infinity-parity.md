# P01.d — Shapes.INFINITY doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 795 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **795/795** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1249/1249** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f`.

Birleşik referans kimliği: `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f`.

Yerel raporlar: `compare-1xrk6zz1` (math debug), `compare-qzmn2buo` (math release), `compare-h3m21imi` (birleşik debug) ve `compare-xnouubh6` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
