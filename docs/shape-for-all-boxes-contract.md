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

Math referans kimliği `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5` oldu. Birleşik altı aile referansı `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
