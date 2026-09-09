# P01.d — AABB sizes doğrulaması

`src/aabb.rs`, Java `AABB.getXsize()`, `getYsize()`, `getZsize()` ve `getSize()` sorgularının çıkarma, toplama ve bölme sırasını uygular. `examples/behavior_adapter.rs` `sizes` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 400 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **400/400** math senaryosunda eşleşti.
- Altı yeni senaryoda birim/dikdörtgen ölçüleri, ters uç normalizasyonu, sıfır uzunluk, signed zero ve büyük toplam yuvarlaması karşılaştırıldı.
- Birleşik altı aile corpus'u **949/949** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `576e0822a563519f05faf72cba233f0591fa0c7e07cf41b43f37b88eecd0b61d`.

Birleşik referans kimliği: `c19eebe34d531a6f87ceeead955aacf7801ad44b42fd02686f35c0b62ea561a4`.

Yerel raporlar: `compare-_41j11nv` (math debug), `compare-xbtyhre_` (math release), `compare-cr7o0h9t` (birleşik debug) ve `compare-ak2lqb__` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB ölçü getter'ı ve mevcut AABB adım adaptörüyle sınırlıdır. BlockPos, Vec3 ve Vector3f move overload'ları [ayrı dilimde tamamlandı](aabb-move-overloads-parity.md); [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
