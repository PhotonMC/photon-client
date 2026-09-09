# P01.d — VoxelShape forAllEdges doğrulaması

`src/shape.rs`, Java `VoxelShape.forAllEdges` callback sırasını `VoxelShape::for_all_edges` ile uygular. Üç `AxisCycle` taraması, dört komşu hücrenin edge koşulu ve ardışık parça birleştirmesi [referans sözleşmesinde](shape-edges-contract.md) tanımlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 935 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **935/935** math senaryosunda eşleşti.
- Üç yeni senaryoda empty callback davranışı, tek kutunun kenar sırası ve birleşik 3B shape'in tüm eksen taraması karşılaştırıldı.
- Birleşik altı aile corpus'u **1437/1437** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a`.

Birleşik referans kimliği: `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a`.

Yerel raporlar: `compare-96_s862r` (math debug), `compare-cbwrzmb5` (math release), `compare-fn36ocz_` (birleşik debug) ve `compare-a_hggp11` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "edges"` shape senaryosuyla ve Java `mergeNeighbors: true` yoluyla sınırlıdır. `forAllEdges` callback'inin `mergeNeighbors: false` iç discrete overload'ı, world collider toplama sonraki kapsamdır.
