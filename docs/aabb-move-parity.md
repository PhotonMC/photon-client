# P01.d — AABB move doğrulaması

`src/aabb.rs`, Java `AABB.move(double,double,double)` metodunun her endpoint'e eksen bazında delta eklenmesini uygular. `examples/behavior_adapter.rs` `move` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 394 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **394/394** math senaryosunda eşleşti.
- Altı yeni senaryoda pozitif/negatif/karışık delta, signed zero, büyük double delta'sı ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1536/1536** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `34b14a75d8c53b3350feba4a21f2e7648537e482ea15a67806d62235270bebad`.

Birleşik referans kimliği: `d50d1ffe555a28907b2748605bdad72312150162a6b1163dca14a521d88ea901`.

Yerel raporlar: `compare-rgmctv_7` (math debug), `compare-w4quj7cd` (math release), `compare-cmzkuxow` (birleşik debug) ve `compare-zlsxov0y` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.move(double,double,double)` ve mevcut AABB adım adaptörüyle sınırlıdır. `BlockPos`, `Vec3` ve `Vector3f` overload'ları için [ayrı kanıt](aabb-move-overloads-parity.md) vardır; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
