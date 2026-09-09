# P01.d — VoxelShape min/max sütun sorguları referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-axis-limits-at-parity.md)). Kapsam, `VoxelShape.min(axis, b, c)` ve `VoxelShape.max(axis, b, c)` overload'larının verilen iki koordinatla seçilen sütundaki dolu hücreleri taramasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:141-161`, `world/phys/shapes/DiscreteVoxelShape.java:98-136` ve `core/AxisCycle.java:1-78` incelendi. `MathOracle`, shape'i Java `Shapes.or(first, tail...)` yoluyla kurar ve her sorguyu doğrudan `min` veya `max` overload'una verir; beklenen koordinatları test içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `axis` sonuç koordinatının eksenidir. `b` ve `c`, bu eksene dik iki eksende sütun seçen koordinatlardır.
- Java `AxisCycle` sırası sütun eksenlerini şöyle seçer:

  | Sonuç ekseni | `b` ekseni | `c` ekseni |
  | --- | --- | --- |
  | X | Y | Z |
  | Y | Z | X |
  | Z | X | Y |

- Her koordinat, mevcut voxel aralığını Java'nın binary search sınırlarıyla seçer. İlk düzlemin altındaki veya son düzlemdeki koordinat geçerli bir hücre sütunu seçmez.
- `min(axis, b, c)`, seçilen sütunda baştan sona taranan ilk dolu hücrenin `axis` başlangıç düzlemini döndürür.
- `max(axis, b, c)`, seçilen sütunda sondan başa taranan son dolu hücrenin `axis` bitiş düzlemini döndürür.
- Sütun koordinatlardan herhangi biri dışındaysa veya sütunda dolu hücre yoksa `min` pozitif sonsuz, `max` negatif sonsuz döndürür.
- Sonuçlar IEEE-754 f64 ham bit string'leriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | Java'da empty olan epsilon altı sliver ve geçerli görünen iki sütun sorgusu |
| disjoint | 1 | Ayrık iki box, üç sonuç ekseni için sütun sorguları ve dış sütun sorgusu |
| negative | 1 | Negatif ve pozitif koordinatları birlikte taşıyan iki box ve üç eksen sorgusu |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "axis_limits_at"`, `shapes` ve `queries` alanlarını kullanır. Her query `op`, `axis`, `b` ve `c` taşır; `b` ile `c` f64 bit string'leridir. Sonuç, sorgu sırasını koruyan `values` dizisinde ham f64 bit string'leri taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliğiyle doğrulandı.

Math debug/release raporları `compare-auuk33g2` ve `compare-w_2b2455`; birleşik debug/release raporları `compare-tjsmsqxw` ve `compare-sks6l8sa` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
