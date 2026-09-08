# P01.d — VoxelShape temel ve OR birleşimi doğrulaması

`src/shape.rs`, Java 26.2 `Shapes` ve `VoxelShape` yollarının bu dilim için gereken box, empty, move, OR join, bounds ve `toAabbs` davranışlarını uygular. Tek shape collision sonucu ayrı [collision kanıtında](shape-collision-parity.md) doğrulanır. `examples/behavior_adapter.rs` aynı çağrı sınırını JSONL testlerine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 345 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **345/345** math senaryosunda eşleşti.
- Beş yeni shape senaryosu; tek box snapshot'ı, `move` sonrası koordinatlar, `1e-7` altı sliver'ın empty olması, ayrık OR kutuları, temas eden OR kutuları ve `joinIsNotEmpty` sonucu üzerinden karşılaştırıldı.
- Birleşik altı aile corpus'u **815/815** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021`.

Birleşik referans kimliği: `49951d8be4544e46974e9f2cf546651f4d0f7ef7a6fbc7ebf9dd5be799d9c402`.

Yerel raporlar: `compare-yvu75qgq` (math debug), `compare-9z90qh6s` (math release), `compare-4qr5yaro` (birleşik debug) ve `compare-2w5gzlh_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik box/OR senaryosuyla sınırlıdır. BooleanOp join sonuçlarının güncel desteklenen alt kümesi için ayrı [BooleanOp kanıtı](shape-boolean-parity.md), tek shape collision için ayrı [collision kanıtı](shape-collision-parity.md) vardır. Tam index merger dalları, çoklu shape collision, yüz/occlusion sorguları, shape clipping ve istemci dünyasındaki collider toplama sonraki dilimlerdir.
