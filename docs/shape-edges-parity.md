# P01.d — VoxelShape forAllEdges doğrulaması

`src/shape.rs`, Java `VoxelShape.forAllEdges` callback sırasını `VoxelShape::for_all_edges` ile uygular. Üç `AxisCycle` taraması, dört komşu hücrenin edge koşulu ve ardışık parça birleştirmesi [referans sözleşmesinde](shape-edges-contract.md) tanımlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 613 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **613/613** math senaryosunda eşleşti.
- Üç yeni senaryoda empty callback davranışı, tek kutunun kenar sırası ve birleşik 3B shape'in tüm eksen taraması karşılaştırıldı.
- Birleşik altı aile corpus'u **1067/1067** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `7f597867c0806ed90f4f27692b375e46994a29d1e67a14a432d2be6b649a5f18`.

Birleşik referans kimliği: `bbf5a13c7b364eb44863c9e8c94841df23c713706e53a57a3b11a552080d0b89`.

Yerel raporlar: `compare-b21h6hpk` (math debug), `compare-69_1nd4e` (math release), `compare-ro657132` (birleşik debug) ve `compare-283vzkto` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "edges"` shape senaryosuyla ve Java `mergeNeighbors: true` yoluyla sınırlıdır. `forAllEdges` callback'inin `mergeNeighbors: false` iç discrete overload'ı, world collider toplama sonraki kapsamdır.
