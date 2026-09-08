# P01.d — VoxelShape temel ve OR birleşimi doğrulaması

`src/shape.rs`, Java 26.2 `Shapes` ve `VoxelShape` yollarının bu dilim için gereken box, empty, move, OR join, bounds ve `toAabbs` davranışlarını uygular. `examples/behavior_adapter.rs` aynı çağrı sınırını JSONL testlerine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 345 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **345/345** math senaryosunda eşleşti.
- Beş yeni shape senaryosu; tek box snapshot'ı, `move` sonrası koordinatlar, `1e-7` altı sliver'ın empty olması, ayrık OR kutuları, temas eden OR kutuları ve `joinIsNotEmpty` sonucu üzerinden karşılaştırıldı.
- Birleşik altı aile corpus'u **803/803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021`.

Birleşik referans kimliği: `9c6f5e56b8630fe1a153fe6933b0c6762ca3b2d4c58d807cd8faab447812bb3d`.

Yerel raporlar: `compare-yvu75qgq` (math debug), `compare-9z90qh6s` (math release), `compare-ry_cgnpg` (birleşik debug) ve `compare-nbf_jf87` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik box/OR senaryosuyla sınırlıdır. `AND`, `ONLY_FIRST` ve `ONLY_SECOND` sonuçları için ayrı [BooleanOp kanıtı](shape-boolean-parity.md) vardır. Tam index merger dalları, diğer BooleanOp işlemleri, collision çözümü, yüz/occlusion sorguları, shape clipping ve istemci dünyasındaki collider toplama sonraki dilimlerdir.
