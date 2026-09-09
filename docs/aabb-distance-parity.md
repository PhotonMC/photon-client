# P01.d — AABB distanceToSqr doğrulaması

`src/aabb.rs`, Java `AABB.distanceToSqr(Vec3)` ve `AABB.distanceToSqr(AABB)` overload'larının eksen başına dış boşluk hesabını ve üç bileşenli kare toplamını uygular. `examples/behavior_adapter.rs` `distance_point` ve `distance_box` adımlarını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 424 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **424/424** math senaryosunda eşleşti.
- Altı yeni senaryoda nokta içi/sınır/dışı sorguları, sonsuzluk ve NaN, örtüşen/değen kutular, diyagonal ayrım ve ters/NaN endpoint'ler karşılaştırıldı.
- Birleşik altı aile corpus'u **1037/1037** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `09c382b6287c06f691d0d698d1c32b5c89a3a02545ec723d11c894ffc006628c`.

Birleşik referans kimliği: `b0cbcb74aa0f40450d03ba7ed23e6155f13776414b6dc51e556a0b02e08fa2c4`.

Yerel raporlar: `compare-zq6owp5b` (math debug), `compare-yw81ny2k` (math release), `compare-be91aw33` (birleşik debug) ve `compare-h5dd6zge` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.distanceToSqr` iki overload'ı ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
