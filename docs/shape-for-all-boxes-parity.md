# P01.d — VoxelShape forAllBoxes doğrulaması

`src/shape.rs`, Java'nın birleşik box taramasını `VoxelShape::for_all_boxes` callback'iyle uygular; `to_aabbs` aynı callback'i kullanır. `examples/behavior_adapter.rs`, callback uçlarını JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 640 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **640/640** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1094/1094** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `boxes` senaryosunda callback çağrı sırası ve altı uç değeri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `81717c09b76bd67677a312941d9eac7dd5776ef9563213a1ab5062f8ef1e32a7`.

Birleşik referans kimliği: `81717c09b76bd67677a312941d9eac7dd5776ef9563213a1ab5062f8ef1e32a7`.

Yerel raporlar: `compare-52kzwns3` (math debug), `compare-8rgxxwte` (math release), `compare-tmxer50a` (birleşik debug) ve `compare-t62jxkbf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box senaryosuyla sınırlıdır. İç discrete `forAllBoxes(..., false)` overload'ı, face taraması ve world collider toplama sonraki kapsamdır.
