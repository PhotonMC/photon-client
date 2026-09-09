# P01.d — Shapes.create(double...) sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-create-coords-parity.md)). Kapsam, `Shapes.create(double...)` overload'ının ham koordinat uç noktalarıyla shape üretme yollarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:52-99`, `BitSetDiscreteVoxelShape.java:23-42` ve `ArrayVoxelShape.java:9-45` incelendi. `MathOracle`, skaler overload'ı doğrudan çağırır; beklenen shape listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- Herhangi bir eksen için `max - min < 1.0E-7` olduğunda sonuç `Shapes.empty()` olur. Ters endpoint'ler bu kontrol nedeniyle ayrıca hata üretmeden empty sonuç verir.
- Üç eksenin endpoint'leri `findBits` içinde `0` ile `3` arasındaki grid aralıklarına oturuyorsa dolu hücreler `CubeVoxelShape` olarak kurulur; hücre sınırları Java `Math.round` ile seçilir.
- Endpoint'lerden biri birim küp/grid sınırlarına sığmazsa `ArrayVoxelShape` döner ve üç eksenin ham `[min, max]` koordinatlarını korur.
- Overload, min/max sırası için `box` gibi bir doğrulama yapmaz. Doğrulama isteyen çağrı `Shapes.box` yoludur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `cube` | Üç eksende 0–1 grid'ine oturan ham endpoint'ler | Cube shape snapshot'ı |
| `reversed` | Ters sıralı ham endpoint'ler | Empty shape snapshot'ı |
| `array` | Birim küp/grid sınırları dışındaki endpoint'ler | Array shape'in box ve bounds snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "create_coords"` ve `box` alanlarını kullanır. Sonuç `empty`, `boxes` ve non-empty durumda `bounds` alanlarını taşır; endpoint'ler f64 bit string'leridir.

## Java yakalama

Math referans kimliği `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7` oldu. Birleşik altı aile referansı `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
