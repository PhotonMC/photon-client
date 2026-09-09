# P01.d — VoxelShape temel ve OR birleşimi referans sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-parity.md)). Kapsam, `Shapes.empty` sonucu, `Shapes.box` ile tek voxel shape üretimi, shape taşıma, iki shape'in `OR` birleşimi ve `joinIsNotEmpty` sorgusunu içerir. Üç veya daha fazla shape'in varargs OR birleşimi ayrı [sözleşmede](shape-or-many-contract.md) ele alınır. VoxelShape.singleEncompassing sonucu ayrı [sözleşmede](shape-single-encompassing-contract.md) ele alınır. VoxelShape min/max sonucu ayrı [sözleşmede](shape-axis-limits-contract.md) ele alınır. VoxelShape min/max sütun sorguları ayrı [sözleşmede](shape-axis-limits-at-contract.md) ele alınır. VoxelShape move overload'ları ayrı [sözleşmede](shape-move-overloads-contract.md) ele alınır. VoxelShape optimize ve getCoords ayrı [sözleşmede](shape-optimize-contract.md) ele alınır. VoxelShape forAllEdges ayrı [sözleşmede](shape-edges-contract.md) ele alınır. `Shapes.join` ve `joinUnoptimized` coordinate plane farkı ayrı [sözleşmede](shape-join-unoptimized-contract.md) ele alınır. VoxelShape forAllBoxes callback sırası ayrı [sözleşmede](shape-for-all-boxes-contract.md) ele alınır. `Shapes.create(AABB)` overload'ı ayrı [sözleşmede](shape-create-aabb-contract.md) ele alınır. `Shapes.create(double...)` overload'ı ayrı [sözleşmede](shape-create-coords-contract.md) ele alınır. `Shapes.INFINITY` sabiti ayrı [sözleşmede](shape-infinity-contract.md) ele alınır. `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` sabitleri ayrı [sözleşmede](shape-epsilon-contract.md) ele alınır. Tek shape collision için ayrı [sözleşme](shape-collision-contract.md), çoklu shape collision için [ayrı sözleşme](shape-collision-many-contract.md), VoxelShape clip için [ayrı sözleşme](shape-clip-contract.md), closestPointTo için [ayrı sözleşme](shape-closest-contract.md), getFaceShape için [ayrı sözleşme](shape-face-contract.md), blockOccludes için [ayrı sözleşme](shape-occlusion-contract.md), mergedFaceOccludes/faceShapeOccludes için [ayrı sözleşme](shape-face-occlusion-contract.md), `Shapes.equal` için [ayrı sözleşme](shape-equal-contract.md), `Shapes.rotate` için [ayrı sözleşme](shape-rotate-contract.md) ve merkez overload için [ayrı sözleşme](shape-rotate-center-contract.md), `Shapes.rotateAllAxis` için [ayrı sözleşme](shape-rotate-all-axis-contract.md), `Shapes.rotateHorizontal` için [ayrı sözleşme](shape-rotate-horizontal-contract.md), `Shapes.rotateAll` için [ayrı sözleşme](shape-rotate-all-contract.md) vardır; diğer yüz/occlusion ve diğer BooleanOp dalları sonraki dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:44-172`, `VoxelShape.java:29-121`, `ArrayVoxelShape.java:9-45`, `CubeVoxelShape.java:7-21`, `BitSetDiscreteVoxelShape.java:112-158` ve `BooleanOp.java:3-21` incelendi. `MathOracle` doğrudan Java shape API'lerini çağırır; beklenen box listelerini yeniden hesaplamaz.

## Davranış sözleşmesi

- `Shapes.box`, min değerlerinin max değerlerini aşmadığını kontrol eder. Herhangi bir eksen aralığı `1e-7` değerinden küçükse sonuç empty shape olur.
- Non-empty shape `bounds` ve `toAabbs` üzerinden gözlenir. Box uçları f64 bitleriyle taşınır; empty shape için box listesi boştur ve bounds eklenmez.
- `move(dx, dy, dz)` shape hücrelerini değiştirmeden her koordinat düzlemini ilgili delta ile taşır.
- `Shapes.or` iki shape'in dolu hücrelerini birleştirir ve komşu full hücreleri Java optimize sırasına göre box listesine çıkarır. `Shapes.or(first, tail...)` tail öğelerini aynı binary OR yolundan soldan sağa geçirir. `AND` ve `ONLY_FIRST`/`ONLY_SECOND` BooleanOp dalları [ayrı sözleşmede](shape-boolean-contract.md) ele alınır. `joinIsNotEmpty` aynı BooleanOp için en az bir dolu sonuç hücresi arar.

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

Shape senaryolarını içeren math referans kimliği `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı da iki JVM'de doğrulandı: `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55`.

Debug ve release Rust adaptörleri math ailesinde 345/345, birleşik corpus'ta 1455/1455 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
