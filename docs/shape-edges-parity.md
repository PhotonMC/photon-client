# P01.d — VoxelShape forAllEdges doğrulaması

`src/shape.rs`, Java `VoxelShape.forAllEdges` callback sırasını `VoxelShape::for_all_edges` ile uygular. Üç `AxisCycle` taraması, dört komşu hücrenin edge koşulu ve ardışık parça birleştirmesi [referans sözleşmesinde](shape-edges-contract.md) tanımlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 655 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **655/655** math senaryosunda eşleşti.
- Üç yeni senaryoda empty callback davranışı, tek kutunun kenar sırası ve birleşik 3B shape'in tüm eksen taraması karşılaştırıldı.
- Birleşik altı aile corpus'u **1109/1109** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Birleşik referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Yerel raporlar: `compare-s2mg_zzb` (math debug), `compare-vw7pifam` (math release), `compare-8ia7c6j_` (birleşik debug) ve `compare-fa02lzlz` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "edges"` shape senaryosuyla ve Java `mergeNeighbors: true` yoluyla sınırlıdır. `forAllEdges` callback'inin `mergeNeighbors: false` iç discrete overload'ı, world collider toplama sonraki kapsamdır.
