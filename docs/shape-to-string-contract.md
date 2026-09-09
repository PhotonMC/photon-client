# P01.d — VoxelShape toString sözleşmesi

Bu dilimde **5 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-to-string-parity.md)). Kapsam, `VoxelShape.toString()` sonucunun empty kısa adı ile dolu shape bounds biçimini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:318-321` ve bounds içindeki `world/phys/AABB.java:433-436` yolları incelendi. `MathOracle`, shape'leri Java üreticileriyle oluşturup doğrudan `toString()` sonucunu kaydeder.

## Davranış sözleşmesi

- Boş shape tam olarak `EMPTY` metnini döndürür.
- Dolu shape `VoxelShape[AABB[minX, minY, minZ] -> [maxX, maxY, maxZ]]` biçimini kullanır.
- Bounds endpoint'leri Java `Double.toString` ile yazılır; `.0`, bilimsel gösterim, `-0.0`, `Infinity`, `-Infinity` ve `NaN` sonuçta görünür.
- Shape metni geometrik kutuların listesini değil, tek encompassing bounds AABB'sini gösterir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `empty` | Threshold altında bir box | `EMPTY` |
| `cube` | Grid tabanlı bounded shape | Bounds metni |
| `scientific` | Küçük/büyük exponent, signed zero ve normal değerler | Java double yazım biçimi |
| `infinity` | Sonsuz endpoint'li shape | Infinity metni |
| `nan` | NaN endpoint'li shape | NaN metni |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "to_string"` ve `shape` alanını kullanır.

## Rust karşılığı

`VoxelShape::to_string_java`, empty kısa yolunu, bounds AABB biçimini ve Java'nın özel double metinlerini üretir. Bilimsel gösterimde Rust'ın kısa round-trip rakamları Java'nın `E`/threshold biçimine uyarlanır; trailing zero'lar sonuçta tutulmaz.
