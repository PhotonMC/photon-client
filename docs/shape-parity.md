# P01.d — VoxelShape temel ve OR birleşimi doğrulaması

`src/shape.rs`, Java 26.2 `Shapes` ve `VoxelShape` yollarının bu dilim için gereken box, empty, move, OR join, bounds ve `toAabbs` davranışlarını uygular. Tek shape collision sonucu ayrı [collision kanıtında](shape-collision-parity.md), çoklu shape collision sonucu [çoklu collision kanıtında](shape-collision-many-parity.md) doğrulanır. `examples/behavior_adapter.rs` aynı çağrı sınırını JSONL testlerine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 345 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **345/345** math senaryosunda eşleşti.
- Beş yeni shape senaryosu; tek box snapshot'ı, `move` sonrası koordinatlar, `1e-7` altı sliver'ın empty olması, ayrık OR kutuları, temas eden OR kutuları ve `joinIsNotEmpty` sonucu üzerinden karşılaştırıldı.
- Birleşik altı aile corpus'u **916/916** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021`.

Birleşik referans kimliği: `f3dac1b46227eaaf38e96f7c59cd9c377a91602c26a477ec26f348affc052a2f`.

Yerel raporlar: `compare-yvu75qgq` (math debug), `compare-9z90qh6s` (math release), `compare-nmul5np1` (birleşik debug) ve `compare-1884mtz2` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik box/OR senaryosuyla sınırlıdır. BooleanOp join sonuçlarının güncel desteklenen alt kümesi için ayrı [BooleanOp kanıtı](shape-boolean-parity.md), tek shape collision için ayrı [collision kanıtı](shape-collision-parity.md) ve çoklu shape collision için [çoklu collision kanıtı](shape-collision-many-parity.md) vardır. Tam index merger dalları, yüz/occlusion sorguları, shape clipping ve istemci dünyasındaki collider toplama sonraki dilimlerdir.
