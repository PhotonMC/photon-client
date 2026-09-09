# P01.d — VoxelShape forAllBoxes sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-for-all-boxes-parity.md)). Kapsam, `VoxelShape.forAllBoxes` callback'inin empty shape, tek box ve birden fazla dolu bölge için Java merge sırasını ve ham f64 uçlarını taşımasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:117-139`, `DiscreteVoxelShape.java:220-222` ve `BitSetDiscreteVoxelShape.java:160` içindeki `forAllBoxes` merge taraması incelendi. `MathOracle`, callback'i doğrudan çalıştırır; beklenen box listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `forAllBoxes(consumer)` shape'in dolu hücrelerini Java'nın `mergeNeighbors: true` yoluyla tarar.
- Her callback çağrısı coordinate grid'inden elde edilen `(minX,minY,minZ,maxX,maxY,maxZ)` uçlarını taşır; callback sırası gözlenebilir davranışın parçasıdır.
- Empty shape callback'i hiç çağırmaz. Komşu dolu hücreler önce Z şeritlerinde, sonra X ve Y boyunca birleştirilir.
- `toAabbs()` aynı callback yolunu liste oluşturarak tüketir; iki API aynı box sırasını paylaşır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `empty` | `1e-7` altı sliver | Boş callback sonucu |
| `box` | Asimetrik tek kutu | Tek callback ve altı ham f64 uç |
| `join` | Üç ayrık 3B kutunun OR birleşimi | Java merge sırasındaki tüm callback uçları |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "boxes"` ve `shapes` alanlarını kullanır. Sonuç `boxes` dizisinde her callback uçlarını altı ham f64 bit string'iyle taşır.

## Java yakalama

Math referans kimliği `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` oldu. Birleşik altı aile referansı `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
