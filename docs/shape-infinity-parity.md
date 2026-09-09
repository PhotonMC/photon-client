# P01.d — Shapes.INFINITY doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 655 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **655/655** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1109/1109** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Birleşik referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Yerel raporlar: `compare-s2mg_zzb` (math debug), `compare-vw7pifam` (math release), `compare-8ia7c6j_` (birleşik debug) ve `compare-fa02lzlz` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
