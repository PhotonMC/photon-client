# P01.d — VoxelShape temel ve OR birleşimi referans sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-parity.md)). Kapsam, `Shapes.empty` sonucu, `Shapes.box` ile tek voxel shape üretimi, shape taşıma, iki shape'in `OR` birleşimi ve `joinIsNotEmpty` sorgusunu içerir. Collision çözümü, yüz/occlusion, ray clipping ve diğer BooleanOp dalları ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:44-172`, `VoxelShape.java:29-121`, `ArrayVoxelShape.java:9-45`, `CubeVoxelShape.java:7-21`, `BitSetDiscreteVoxelShape.java:112-158` ve `BooleanOp.java:3-21` incelendi. `MathOracle` doğrudan Java shape API'lerini çağırır; beklenen box listelerini yeniden hesaplamaz.

## Davranış sözleşmesi

- `Shapes.box`, min değerlerinin max değerlerini aşmadığını kontrol eder. Herhangi bir eksen aralığı `1e-7` değerinden küçükse sonuç empty shape olur.
- Non-empty shape `bounds` ve `toAabbs` üzerinden gözlenir. Box uçları f64 bitleriyle taşınır; empty shape için box listesi boştur ve bounds eklenmez.
- `move(dx, dy, dz)` shape hücrelerini değiştirmeden her koordinat düzlemini ilgili delta ile taşır.
- `Shapes.or` iki shape'in dolu hücrelerini birleştirir ve komşu full hücreleri Java optimize sırasına göre box listesine çıkarır. `AND` ve `ONLY_FIRST`/`ONLY_SECOND` BooleanOp dalları [ayrı sözleşmede](shape-boolean-contract.md) ele alınır. `joinIsNotEmpty` aynı BooleanOp için en az bir dolu sonuç hücresi arar.

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

Shape senaryolarını içeren math referans kimliği `eeccf6a1ab099f28829b45e5c6f977e0ebe570005108d43a151a094f55b9f021` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı da iki JVM'de doğrulandı: `de3c26ebb4097aa764d25839e62f20bee934c42bb19318718b2b46242990a108`.

Debug ve release Rust adaptörleri math ailesinde 345/345, birleşik corpus'ta 807/807 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
