# P01.d — VoxelShape forAllBoxes doğrulaması

`src/shape.rs`, Java'nın birleşik box taramasını `VoxelShape::for_all_boxes` callback'iyle uygular; `to_aabbs` aynı callback'i kullanır. `examples/behavior_adapter.rs`, callback uçlarını JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1653 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `boxes` senaryosunda callback çağrı sırası ve altı uç değeri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Yerel raporlar: `compare-ceusm4y4` (math debug), `compare-hconpy1a` (math release), `compare-rhjx0z7e` (birleşik debug) ve `compare-jtl411eu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box senaryosuyla sınırlıdır. İç discrete `forAllBoxes(..., false)` overload'ı, face taraması ve world collider toplama sonraki kapsamdır.
