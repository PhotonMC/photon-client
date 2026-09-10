# P01.d — VoxelShape optimize ve getCoords referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-optimize-parity.md)). Kapsam, `VoxelShape.optimize()` ile shape'in Java kutu iterasyonu üzerinden yeniden kurulmasını ve `getCoords` coordinate plane sonuçlarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:71,96-100,117-139`, `DiscreteVoxelShape.java:220-221`, `BitSetDiscreteVoxelShape.java:160-208` ve `Shapes.java:139-172` incelendi. `MathOracle`, optimize öncesi ve sonrası `getCoords` listelerini doğrudan Java shape nesnesinden okur.

## Davranış sözleşmesi

- `getCoords(axis)` eksenin artan coordinate plane listesini döndürür; empty shape'in her eksende tek `0.0` düzlemi vardır.
- `optimize()` `forAllBoxes` çağrısının Java merge sırasındaki her kutusunu `Shapes.box` ile yeniden üretir ve bunları `Shapes.joinUnoptimized(..., OR)` yoluyla soldan sağa birleştirir.
- Optimize sonucu dolu hücre işgalini korur; komşu kutuların gereksiz ara coordinate plane'leri kaldırılır, ayrık kutuların geometriyi tanımlayan düzlemleri korunur.
- Empty shape optimize edildiğinde yeni kutu üretmeden empty shape döner.
- Coordinate değerleri Java double sonuçları olarak, karşılaştırma ham f64 bitleriyle yapılır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | `1e-7` altı aralığın optimize edilmesi ve üç eksenin koordinat listeleri |
| temas eden satır | 1 | X ekseninde üç komşu kutu; optimize öncesi beş, sonrası iki X düzlemi |
| ayrık | 1 | Üç farklı 3B kutu; optimize sonrası gerekli birleşik düzlemler |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "optimize"` ve `shapes` alanlarını kullanır. Java oracle, girdileri önce `Shapes.joinUnoptimized(..., OR)` ile kurar; `events` ilk shape ve `shape.optimize()` snapshot'larını taşır. Her snapshot `empty`, `boxes`, `bounds` ve X/Y/Z `coords` listelerini içerir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğiyle doğrulandı.

Math debug/release raporları `compare-xljwmsb7` ve `compare-s_oqvkq3`; birleşik debug/release raporları `compare-uf5y7y3a` ve `compare-wrj9a6ex` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
