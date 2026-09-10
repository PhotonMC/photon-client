# P01.d — Shapes.INFINITY doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1743 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Yerel raporlar: `compare-q3ryrs_k` (math debug), `compare-xtfhknri` (math release), `compare-ag5oxl30` (birleşik debug) ve `compare-fkrk2srf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
