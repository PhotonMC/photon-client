# P01.d — VoxelShape move overload'ları doğrulaması

`src/shape.rs`, Java `VoxelShape.move(Vec3)` ve `move(Vec3i)` çağrılarını `VoxelShape::move_by_vec3` ve `VoxelShape::move_by_block_pos` ile uygular; iki yöntem de mevcut scalar `move_by` yoluna bağlanır. `examples/behavior_adapter.rs` `move_vec3` ve `move_block_pos` shape adımlarını bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-move-overloads-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 680 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **680/680** math senaryosunda eşleşti.
- Üç yeni senaryoda Vec3 double delta'sı, Vec3i integer delta'sı ve empty shape üzerinde iki overload zinciri karşılaştırıldı.
- Birleşik altı aile corpus'u **1134/1134** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7`.

Birleşik referans kimliği: `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7`.

Yerel raporlar: `compare-yr90fi_v` (math debug), `compare-a6r_92in` (math release), `compare-aiddld1f` (birleşik debug) ve `compare-micscpjd` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "single"` shape senaryosuyla ve Vec3/Vec3i move adımlarıyla sınırlıdır. `optimize`, `forAllEdges`, doğrudan koordinat listeleri ve istemci hareketindeki shape taşıma zinciri sonraki kapsamdır. Scalar `move(double,double,double)` davranışı mevcut [temel shape kanıtında](shape-parity.md) yer alır.
