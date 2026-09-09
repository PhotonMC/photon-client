# P01.d — AABB move doğrulaması

`src/aabb.rs`, Java `AABB.move(double,double,double)` metodunun her endpoint'e eksen bazında delta eklenmesini uygular. `examples/behavior_adapter.rs` `move` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 394 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **394/394** math senaryosunda eşleşti.
- Altı yeni senaryoda pozitif/negatif/karışık delta, signed zero, büyük double delta'sı ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1052/1052** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `34b14a75d8c53b3350feba4a21f2e7648537e482ea15a67806d62235270bebad`.

Birleşik referans kimliği: `79c0a319fee88ab66551f0d6e482274fbf9e528c52ae097bb43853784b4e846a`.

Yerel raporlar: `compare-rgmctv_7` (math debug), `compare-w4quj7cd` (math release), `compare-witt8p3m` (birleşik debug) ve `compare-pabdz7bu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.move(double,double,double)` ve mevcut AABB adım adaptörüyle sınırlıdır. `BlockPos`, `Vec3` ve `Vector3f` overload'ları için [ayrı kanıt](aabb-move-overloads-parity.md) vardır; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
