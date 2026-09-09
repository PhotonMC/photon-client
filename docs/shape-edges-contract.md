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

Bu dilimin bulunduğu güncel math referansı `88893f7d2054eb1f0f13242d6d887e1ae0d0e1dcb1a99c08a190cff768f8b841` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `fa080fc05d94f4eec8044ce6e4989a288e586b7d5065e1926ba18b75170a3bdb` kimliğiyle doğrulandı.

Math debug/release raporları `compare-p21yz9ko` ve `compare-tvqo_c61`; birleşik debug/release raporları `compare-fgeq_2hx` ve `compare-b0c437h_` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
