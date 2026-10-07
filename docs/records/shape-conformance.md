# P01.d — VoxelShape temel ve OR birleşimi referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.empty` sonucu, `Shapes.box` ile tek voxel shape üretimi, shape taşıma, iki shape'in `OR` birleşimi ve `joinIsNotEmpty` sorgusunu içerir. Üç veya daha fazla shape'in varargs OR birleşimi ayrı [sözleşmede](shape-or-many-conformance.md) ele alınır. VoxelShape.singleEncompassing sonucu ayrı [sözleşmede](shape-single-encompassing-conformance.md) ele alınır. VoxelShape min/max sonucu ayrı [sözleşmede](shape-axis-limits-conformance.md) ele alınır. VoxelShape min/max sütun sorguları ayrı [sözleşmede](shape-axis-limits-at-conformance.md) ele alınır. VoxelShape move overload'ları ayrı [sözleşmede](shape-move-overloads-conformance.md) ele alınır. VoxelShape optimize ve getCoords ayrı [sözleşmede](shape-optimize-conformance.md) ele alınır. VoxelShape forAllEdges ayrı [sözleşmede](shape-edges-conformance.md) ele alınır. `Shapes.join` ve `joinUnoptimized` coordinate plane farkı ayrı [sözleşmede](shape-join-unoptimized-conformance.md) ele alınır. VoxelShape forAllBoxes callback sırası ayrı [sözleşmede](shape-for-all-boxes-conformance.md) ele alınır. `Shapes.create(AABB)` overload'ı ayrı [sözleşmede](shape-create-aabb-conformance.md) ele alınır. `Shapes.create(double...)` overload'ı ayrı [sözleşmede](shape-create-coords-conformance.md) ele alınır. `Shapes.INFINITY` sabiti ayrı [sözleşmede](shape-infinity-conformance.md) ele alınır. `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitleri ayrı [sözleşmede](shape-epsilon-conformance.md) ele alınır. Tek shape collision için ayrı [sözleşme](shape-collision-conformance.md), çoklu shape collision için [ayrı sözleşme](shape-collision-many-conformance.md), VoxelShape clip için [ayrı sözleşme](shape-clip-conformance.md), closestPointTo için [ayrı sözleşme](shape-closest-conformance.md), getFaceShape için [ayrı sözleşme](shape-face-conformance.md), blockOccludes için [ayrı sözleşme](shape-occlusion-conformance.md), mergedFaceOccludes/faceShapeOccludes için [ayrı sözleşme](shape-face-occlusion-conformance.md), `Shapes.equal` için [ayrı sözleşme](shape-equal-conformance.md), `Shapes.rotate` için [ayrı sözleşme](shape-rotate-conformance.md) ve merkez overload için [ayrı sözleşme](shape-rotate-center-conformance.md), `Shapes.rotateAllAxis` için [ayrı sözleşme](shape-rotate-all-axis-conformance.md), `Shapes.rotateHorizontal` için [ayrı sözleşme](shape-rotate-horizontal-conformance.md), `Shapes.rotateAll` için [ayrı sözleşme](shape-rotate-all-conformance.md) vardır; diğer yüz/occlusion ve diğer BooleanOp dalları sonraki dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:44-172`, `VoxelShape.java:29-121`, `ArrayVoxelShape.java:9-45`, `CubeVoxelShape.java:7-21`, `BitSetDiscreteVoxelShape.java:112-158` ve `BooleanOp.java:3-21` incelendi. `MathOracle` doğrudan Java shape API'lerini çağırır; beklenen box listelerini yeniden hesaplamaz.

## Davranış sözleşmesi

- `Shapes.box`, min değerlerinin max değerlerini aşmadığını kontrol eder. Herhangi bir eksen aralığı `1e-7` değerinden küçükse sonuç empty shape olur.
- Non-empty shape `bounds` ve `toAabbs` üzerinden gözlenir. Box uçları f64 bitleriyle taşınır; empty shape için box listesi boştur ve bounds eklenmez.
- `move(dx, dy, dz)` shape hücrelerini değiştirmeden her koordinat düzlemini ilgili delta ile taşır.
- `Shapes.or` iki shape'in dolu hücrelerini birleştirir ve komşu full hücreleri Java optimize sırasına göre box listesine çıkarır. `Shapes.or(first, tail...)` tail öğelerini aynı binary OR yolundan soldan sağa geçirir. `AND` ve `ONLY_FIRST`/`ONLY_SECOND` BooleanOp dalları [ayrı sözleşmede](shape-boolean-conformance.md) ele alınır. `joinIsNotEmpty` aynı BooleanOp için en az bir dolu sonuç hücresi arar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| tek shape | 3 | Birim block, aralık dışı koordinatları taşıma ve `1e-7` altı sliver |
| OR birleşimi | 2 | Ayrık ve yalnızca yüzeyden temas eden iki kutu |

`family: "math"`, `operation: "shape"` girdileridir. Tek shape sonucu `events` içinde, birleşim sonucu `shape` ve `join_is_not_empty` alanlarında taşınır. Shape snapshot'ı `empty`, `boxes` ve non-empty durumda `bounds` alanlarını içerir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Shape senaryolarını içeren math referans kimliği `eeccf6a1ab099f28829b45e5c6f1019e0ebe570005108d43a151a094f55b9f021` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı da iki JVM'de doğrulandı: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Debug ve release Rust adaptörleri math ailesinde 345/345, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java 26.2 `Shapes` ve `VoxelShape` yollarının bu dilim için gereken box, empty, move, iki shape OR join, bounds ve `toAabbs` davranışlarını uygular. Üç veya daha fazla shape'in varargs OR sonucu ayrı [varargs kanıtında](shape-or-many-conformance.md) doğrulanır. VoxelShape.singleEncompassing sonucu ayrı [kanıtta](shape-single-encompassing-conformance.md) doğrulanır. VoxelShape min/max sonucu ayrı [kanıtta](shape-axis-limits-conformance.md) doğrulanır. VoxelShape min/max sütun sorguları ayrı [kanıtta](shape-axis-limits-at-conformance.md) doğrulanır. VoxelShape move overload'ları ayrı [kanıtta](shape-move-overloads-conformance.md) doğrulanır. VoxelShape optimize ve getCoords ayrı [kanıtta](shape-optimize-conformance.md) doğrulanır. VoxelShape forAllBoxes ayrı [kanıtta](shape-for-all-boxes-conformance.md), `Shapes.create(AABB)` overload'ı ayrı [kanıtta](shape-create-aabb-conformance.md), `Shapes.create(double...)` overload'ı ayrı [kanıtta](shape-create-coords-conformance.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-conformance.md), `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitleri ayrı [kanıtta](shape-epsilon-conformance.md) ve VoxelShape forAllEdges ayrı [kanıtta](shape-edges-conformance.md) doğrulanır. VoxelShape clip hit sonucu ayrı [clip kanıtında](shape-clip-conformance.md), closestPointTo sonucu ayrı [closest point kanıtında](shape-closest-conformance.md), getFaceShape sonucu ayrı [face kanıtında](shape-face-conformance.md), blockOccludes sonucu ayrı [occlusion kanıtında](shape-occlusion-conformance.md), mergedFaceOccludes/faceShapeOccludes sonuçları ayrı [face occlusion kanıtında](shape-face-occlusion-conformance.md), `Shapes.equal` sonucu ayrı [eşitlik kanıtında](shape-equal-conformance.md), `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-conformance.md) ve merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-conformance.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-conformance.md), merkez overload sonucu [yatay eksen merkez kanıtında](shape-rotate-horizontal-axis-center-conformance.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-conformance.md), merkez overload sonucu [all-axis merkez kanıtında](shape-rotate-all-axis-center-conformance.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-conformance.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-conformance.md), merkez overload sonucu [rotateAll merkez kanıtında](shape-rotate-all-center-conformance.md), Shapes.rotateAttachFace sonucu [attach-face kanıtında](shape-rotate-attach-face-conformance.md), Shapes.rotateHorizontal initial sonucu [initial kanıtında](shape-rotate-horizontal-initial-conformance.md), Shapes.rotateAll initial sonucu [all-direction initial kanıtında](shape-rotate-all-initial-conformance.md), Shapes.rotateAttachFace initial sonucu [attach-face initial kanıtında](shape-rotate-attach-face-initial-conformance.md) doğrulanır. Tek shape collision sonucu ayrı [collision kanıtında](shape-collision-conformance.md), çoklu shape collision sonucu [çoklu collision kanıtında](shape-collision-many-conformance.md) doğrulanır. `examples/behavior_adapter.rs` aynı çağrı sınırını JSONL testlerine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 345 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **345/345** math senaryosunda eşleşti.
- Beş yeni shape senaryosu; tek box snapshot'ı, `move` sonrası koordinatlar, `1e-7` altı sliver'ın empty olması, ayrık OR kutuları, temas eden OR kutuları ve `joinIsNotEmpty` sonucu üzerinden karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `eeccf6a1ab099f28829b45e5c6f1019e0ebe570005108d43a151a094f55b9f021`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-yvu75qgq` (math debug), `compare-9z90qh6s` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt beş sentetik box/OR senaryosuyla sınırlıdır. BooleanOp join sonuçlarının güncel desteklenen alt kümesi için ayrı [BooleanOp kanıtı](shape-boolean-conformance.md), tek shape collision için ayrı [collision kanıtı](shape-collision-conformance.md) ve çoklu shape collision için [çoklu collision kanıtı](shape-collision-many-conformance.md) vardır. Tam index merger dalları, yüz/occlusion sorguları ve istemci dünyasındaki collider toplama sonraki dilimlerdir.
