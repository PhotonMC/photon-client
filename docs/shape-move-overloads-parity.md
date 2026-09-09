# P01.d — VoxelShape move overload'ları doğrulaması

`src/shape.rs`, Java `VoxelShape.move(Vec3)` ve `move(Vec3i)` çağrılarını `VoxelShape::move_by_vec3` ve `VoxelShape::move_by_block_pos` ile uygular; iki yöntem de mevcut scalar `move_by` yoluna bağlanır. `examples/behavior_adapter.rs` `move_vec3` ve `move_block_pos` shape adımlarını bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-move-overloads-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 947 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **947/947** math senaryosunda eşleşti.
- Üç yeni senaryoda Vec3 double delta'sı, Vec3i integer delta'sı ve empty shape üzerinde iki overload zinciri karşılaştırıldı.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363`.

Birleşik referans kimliği: `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363`.

Yerel raporlar: `compare-auuk33g2` (math debug), `compare-w_2b2455` (math release), `compare-tjsmsqxw` (birleşik debug) ve `compare-sks6l8sa` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "single"` shape senaryosuyla ve Vec3/Vec3i move adımlarıyla sınırlıdır. `optimize`, `forAllEdges`, doğrudan koordinat listeleri ve istemci hareketindeki shape taşıma zinciri sonraki kapsamdır. Scalar `move(double,double,double)` davranışı mevcut [temel shape kanıtında](shape-parity.md) yer alır.
