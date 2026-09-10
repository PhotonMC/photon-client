# P01.d — VoxelShape forAllBoxes doğrulaması

`src/shape.rs`, Java'nın birleşik box taramasını `VoxelShape::for_all_boxes` callback'iyle uygular; `to_aabbs` aynı callback'i kullanır. `examples/behavior_adapter.rs`, callback uçlarını JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1638 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `boxes` senaryosunda callback çağrı sırası ve altı uç değeri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Yerel raporlar: `compare-ep8fiu6` (math debug), `compare-z9iwcisf` (math release), `compare-a21c2t4m` (birleşik debug) ve `compare-uwg2_ewv` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box senaryosuyla sınırlıdır. İç discrete `forAllBoxes(..., false)` overload'ı, face taraması ve world collider toplama sonraki kapsamdır.
