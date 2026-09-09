# P01.d — VoxelShape forAllEdges doğrulaması

`src/shape.rs`, Java `VoxelShape.forAllEdges` callback sırasını `VoxelShape::for_all_edges` ile uygular. Üç `AxisCycle` taraması, dört komşu hücrenin edge koşulu ve ardışık parça birleştirmesi [referans sözleşmesinde](shape-edges-contract.md) tanımlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Üç yeni senaryoda empty callback davranışı, tek kutunun kenar sırası ve birleşik 3B shape'in tüm eksen taraması karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Birleşik referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug) ve `compare-4_se5ekn` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "edges"` shape senaryosuyla ve Java `mergeNeighbors: true` yoluyla sınırlıdır. `forAllEdges` callback'inin `mergeNeighbors: false` iç discrete overload'ı, world collider toplama sonraki kapsamdır.
