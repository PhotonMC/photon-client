# P01.d — VoxelShape move overload'ları doğrulaması

`src/shape.rs`, Java `VoxelShape.move(Vec3)` ve `move(Vec3i)` çağrılarını `VoxelShape::move_by_vec3` ve `VoxelShape::move_by_block_pos` ile uygular; iki yöntem de mevcut scalar `move_by` yoluna bağlanır. `examples/behavior_adapter.rs` `move_vec3` ve `move_block_pos` shape adımlarını bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-move-overloads-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 929 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **929/929** math senaryosunda eşleşti.
- Üç yeni senaryoda Vec3 double delta'sı, Vec3i integer delta'sı ve empty shape üzerinde iki overload zinciri karşılaştırıldı.
- Birleşik altı aile corpus'u **1431/1431** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88`.

Birleşik referans kimliği: `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88`.

Yerel raporlar: `compare-eiqs7rub` (math debug), `compare-t3kloypj` (math release), `compare-wfqzv7gl` (birleşik debug) ve `compare-_to6641m` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "single"` shape senaryosuyla ve Vec3/Vec3i move adımlarıyla sınırlıdır. `optimize`, `forAllEdges`, doğrudan koordinat listeleri ve istemci hareketindeki shape taşıma zinciri sonraki kapsamdır. Scalar `move(double,double,double)` davranışı mevcut [temel shape kanıtında](shape-parity.md) yer alır.
