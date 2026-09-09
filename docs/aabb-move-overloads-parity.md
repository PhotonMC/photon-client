# P01.d — AABB move overload doğrulaması

`src/aabb.rs`, Java `AABB.move(BlockPos)`, `AABB.move(Vec3)` ve `AABB.move(Vector3f)` yollarını sırasıyla integer, double ve float bileşenleriyle taşıyıp ortak scalar move hesabına bağlar. `examples/behavior_adapter.rs` üç overload adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 442 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **442/442** math senaryosunda eşleşti.
- Altı yeni senaryoda BlockPos i32 sınırları, Vec3 özel değerleri, Vector3f f32 hassasiyeti, signed zero ve overload zincirleri karşılaştırıldı.
- Birleşik altı aile corpus'u **916/916** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e7ae22f078eb9523961cde345ba3ea72bd43b52904e0c750f321f6b4663fd1cd`.

Birleşik referans kimliği: `f3dac1b46227eaaf38e96f7c59cd9c377a91602c26a477ec26f348affc052a2f`.

Yerel raporlar: `compare-4ymeo58v` (math debug), `compare-x3p8pg_j` (math release), `compare-nmul5np1` (birleşik debug) ve `compare-1884mtz2` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.move(BlockPos)`, `AABB.move(Vec3)`, `AABB.move(Vector3f)` ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; AABB.of(BoundingBox), diğer clip overload'ları ve çoklu AABB clip sonraki dilimlerdir.
