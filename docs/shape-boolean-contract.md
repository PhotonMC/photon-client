# P01.d — VoxelShape BooleanOp join sözleşmesi

Bu dilimin **4 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-boolean-parity.md)). Kapsam, `Shapes.join` ve `Shapes.joinIsNotEmpty` çağrılarında `BooleanOp.AND`, `BooleanOp.ONLY_FIRST` ve `BooleanOp.ONLY_SECOND` dallarını içerir. OR, box üretimi ve taşıma için [temel shape sözleşmesi](shape-contract.md) geçerlidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/BooleanOp.java:3-21`, `Shapes.java:127-232` ve `BitSetDiscreteVoxelShape.java:112-158` incelendi. `MathOracle` BooleanOp değerini Java API'sine geçirir; beklenen kutuları yeniden hesaplamaz.

## Davranış sözleşmesi

- `AND` yalnızca iki girdinin de dolu olduğu ortak hücreleri taşır.
- `ONLY_FIRST`, ilk girdinin ikinciyle paylaşmadığı hücreleri; `ONLY_SECOND` ikinci girdinin ilk girdinin paylaşmadığı hücreleri taşır.
- Join sonrası komşu dolu hücreler Java `optimize`/`toAabbs` sırasına göre AABB listesine çıkarılır. Hiç dolu hücre kalmadığında snapshot `empty: true`, boş `boxes` listesi ve `join_is_not_empty: false` taşır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| AND | 2 | Örtüşen kutular ve ayrık kutular |
| ONLY_FIRST | 1 | İlk kutunun ortasından çıkarılan ikinci kutu |
| ONLY_SECOND | 1 | Ayrık kutularda ikinci taraf |

`family: "math"`, `operation: "shape"`, `kind: "join"` girdileridir. `op` alanı `and`, `only_first` veya `only_second` olur. Sonuç `shape` snapshot'ı ile `join_is_not_empty` alanını içerir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

BooleanOp senaryolarını içeren math referans kimliği `3fbfb7e1e08bca037f422300c2080c84c0ab4608f3cd9c710e966f1c27d5085d` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `9c6f5e56b8630fe1a153fe6933b0c6762ca3b2d4c58d807cd8faab447812bb3d` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 349/349, birleşik corpus'ta 803/803 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
