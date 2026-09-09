# P01.d — VoxelShape move overload'ları doğrulaması

`src/shape.rs`, Java `VoxelShape.move(Vec3)` ve `move(Vec3i)` çağrılarını `VoxelShape::move_by_vec3` ve `VoxelShape::move_by_block_pos` ile uygular; iki yöntem de mevcut scalar `move_by` yoluna bağlanır. `examples/behavior_adapter.rs` `move_vec3` ve `move_block_pos` shape adımlarını bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-move-overloads-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1249 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1249/1249** math senaryosunda eşleşti.
- Üç yeni senaryoda Vec3 double delta'sı, Vec3i integer delta'sı ve empty shape üzerinde iki overload zinciri karşılaştırıldı.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Yerel raporlar: `compare-dj1scyn3` (math debug), `compare-1g72i2us` (math release), `compare-f0bjlm2c` (birleşik debug) ve `compare-eaqfihsj` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "single"` shape senaryosuyla ve Vec3/Vec3i move adımlarıyla sınırlıdır. `optimize`, `forAllEdges`, doğrudan koordinat listeleri ve istemci hareketindeki shape taşıma zinciri sonraki kapsamdır. Scalar `move(double,double,double)` davranışı mevcut [temel shape kanıtında](shape-parity.md) yer alır.
