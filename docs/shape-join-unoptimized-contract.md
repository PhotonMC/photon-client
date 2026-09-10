# P01.d — VoxelShape join ve joinUnoptimized sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-join-unoptimized-parity.md)). Kapsam, `Shapes.join` sonucunun `optimize` adımından geçmesini, `Shapes.joinUnoptimized` sonucunun birleşik coordinate plane'lerini korumasını ve empty shape fast path'lerinin diğer shape'in koordinatlarını taşımasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:135-173`, `VoxelShape.java:96-100,117-139` ve `DiscreteVoxelShape.java` içindeki box taraması incelendi. `MathOracle`, iki Java metodunu doğrudan çağırır ve her sonucun box, bounds ve `getCoords` değerlerini ham f64 bitleriyle kaydeder.

## Davranış sözleşmesi

- `Shapes.join(first, second, op)`, önce `joinUnoptimized` ile sonucu üretir, ardından `VoxelShape.optimize()` çağırır.
- `Shapes.joinUnoptimized`, BooleanOp sonucunu birleşik coordinate grid'inin her hücresine uygular; girdilerden biri empty ise `op` yalnızca diğer tarafı önemsiyorsa diğer shape'i, aksi halde empty shape'i döndürür.
- `joinUnoptimized` sonucu, dolu hücreler sonradan birleşebilse bile girdilerin coordinate plane'lerini korur. `optimize` box iterasyonu ile yeniden kurulduğundan touching kutular ortak plane'e indirgenebilir.
- Empty sonuçta Java'nın empty coordinate listeleri korunur; non-empty sonuçta koordinat düzlemleri ve box sırası gözlenebilir davranışın parçasıdır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `touching` | X ekseninde bitişen iki kutu, `OR` | `joinUnoptimized` ara `.25` plane'ini korur; `join` optimize edip `.5` sınırlarına iner |
| `hole` | Birim kutudan orta X dilimi çıkarılır, `ONLY_FIRST` | İki sonuç da iki dış kutuyu ve tüm birleşik coordinate plane'lerini taşır |
| `empty-first` | Sliver empty shape ile uzak bir kutu, `OR` | Her iki sonuç da ikinci shape'in koordinatlarını doğrudan taşır |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "join_coords"` biçimindedir. `unoptimized` ve `optimized` alanları `empty`, `boxes`, `bounds` ve `coords` snapshot'larını içerir.

## Java yakalama

Math referans kimliği `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11` oldu. Birleşik altı aile referansı `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
