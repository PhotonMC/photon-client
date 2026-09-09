# P01.d — VoxelShape temel ve OR birleşimi doğrulaması

`src/shape.rs`, Java 26.2 `Shapes` ve `VoxelShape` yollarının bu dilim için gereken box, empty, move, OR join, bounds ve `toAabbs` davranışlarını uygular. VoxelShape clip hit sonucu ayrı [clip kanıtında](shape-clip-parity.md), closestPointTo sonucu ayrı [closest point kanıtında](shape-closest-parity.md), getFaceShape sonucu ayrı [face kanıtında](shape-face-parity.md), blockOccludes sonucu ayrı [occlusion kanıtında](shape-occlusion-parity.md), mergedFaceOccludes/faceShapeOccludes sonuçları ayrı [face occlusion kanıtında](shape-face-occlusion-parity.md), `Shapes.equal` sonucu ayrı [eşitlik kanıtında](shape-equal-parity.md), `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md) ve merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md), merkez overload sonucu [rotateAll merkez kanıtında](shape-rotate-all-center-parity.md) doğrulanır. Tek shape collision sonucu ayrı [collision kanıtında](shape-collision-parity.md), çoklu shape collision sonucu [çoklu collision kanıtında](shape-collision-many-parity.md) doğrulanır. `examples/behavior_adapter.rs` aynı çağrı sınırını JSONL testlerine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 345 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **345/345** math senaryosunda eşleşti.
- Beş yeni shape senaryosu; tek box snapshot'ı, `move` sonrası koordinatlar, `1e-7` altı sliver'ın empty olması, ayrık OR kutuları, temas eden OR kutuları ve `joinIsNotEmpty` sonucu üzerinden karşılaştırıldı.
- Birleşik altı aile corpus'u **1013/1013** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021`.

Birleşik referans kimliği: `8bd229a580fc01d7c478c33923afb04c0fee7803869b7423f2d3ad4587821b9f`.

Yerel raporlar: `compare-yvu75qgq` (math debug), `compare-9z90qh6s` (math release), `compare-n_x78ct2` (birleşik debug) ve `compare-pzl7bv8b` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik box/OR senaryosuyla sınırlıdır. BooleanOp join sonuçlarının güncel desteklenen alt kümesi için ayrı [BooleanOp kanıtı](shape-boolean-parity.md), tek shape collision için ayrı [collision kanıtı](shape-collision-parity.md) ve çoklu shape collision için [çoklu collision kanıtı](shape-collision-many-parity.md) vardır. Tam index merger dalları, yüz/occlusion sorguları ve istemci dünyasındaki collider toplama sonraki dilimlerdir.
