# P01.d — VoxelShape forAllEdges referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-edges-parity.md)). Kapsam, `VoxelShape.forAllEdges` çağrısının dolu hücre sınır kenarlarını Java eksen çevrimi ve merge sırasıyla ziyaret etmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:102-115`, `DiscreteVoxelShape.java:62-74,160-218`, `AxisCycle.java:3-65` ve `Direction.Axis` seçimi incelendi. `MathOracle`, Java callback'inin aldığı altı double koordinatı doğrudan JSON kenar listesine taşır.

## Davranış sözleşmesi

- `forAllEdges` boş shape için callback çağırmaz.
- Kenarlar X, Y ve Z yönlerini temsil eden üç `AxisCycle` taramasında Java sırasıyla ziyaret edilir.
- Her coordinate plane noktasında dört komşu hücrenin `fullSectors` sayısı ve `oddSectors` paritesi hesaplanır; Java koşulunu sağlayan ardışık kenar parçaları tek callback çağrısında birleştirilir.
- Callback uçları shape'in coordinate plane listelerinden okunur ve `(x1, y1, z1, x2, y2, z2)` sırası korunur.
- Koordinatlar Java double sonuçları olarak, karşılaştırma ham f64 bitleriyle yapılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | `1e-7` altı aralıkla empty shape |
| tek kutu | 1 | Asimetrik koordinatlı tek box'ın 12 kenarı |
| birleşik | 1 | Üç 3B box'ın OR birleşimi ve callback sırası |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "edges"` ve `shapes` alanlarını kullanır. Sonuç `edges` altında, her kenarı altı f64 bit string'i taşıyan sıralı bir liste olarak döndürülür.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` kimliğiyle doğrulandı.

Math debug/release raporları `compare-yy9k89p5` ve `compare-og6yrygd`; birleşik debug/release raporları `compare-downo9zd` ve `compare-66zm55sg` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
