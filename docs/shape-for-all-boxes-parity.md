# P01.d — VoxelShape forAllBoxes doğrulaması

`src/shape.rs`, Java'nın birleşik box taramasını `VoxelShape::for_all_boxes` callback'iyle uygular; `to_aabbs` aynı callback'i kullanır. `examples/behavior_adapter.rs`, callback uçlarını JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 695 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **695/695** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1149/1149** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `boxes` senaryosunda callback çağrı sırası ve altı uç değeri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Birleşik referans kimliği: `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5`.

Yerel raporlar: `compare-z24w3ncm` (math debug), `compare-ue0l25nl` (math release), `compare-oiqblje5` (birleşik debug) ve `compare-pl0oao3t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box senaryosuyla sınırlıdır. İç discrete `forAllBoxes(..., false)` overload'ı, face taraması ve world collider toplama sonraki kapsamdır.
