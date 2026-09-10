# P01.d — VoxelShape forAllEdges doğrulaması

`src/shape.rs`, Java `VoxelShape.forAllEdges` callback sırasını `VoxelShape::for_all_edges` ile uygular. Üç `AxisCycle` taraması, dört komşu hücrenin edge koşulu ve ardışık parça birleştirmesi [referans sözleşmesinde](shape-edges-contract.md) tanımlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1478 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1478/1478** math senaryosunda eşleşti.
- Üç yeni senaryoda empty callback davranışı, tek kutunun kenar sırası ve birleşik 3B shape'in tüm eksen taraması karşılaştırıldı.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug) ve `compare-cvif6gdr` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "edges"` shape senaryosuyla ve Java `mergeNeighbors: true` yoluyla sınırlıdır. `forAllEdges` callback'inin `mergeNeighbors: false` iç discrete overload'ı, world collider toplama sonraki kapsamdır.
