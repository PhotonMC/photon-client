# P01.d — VoxelShape min/max referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-axis-limits-parity.md)). Kapsam, bir VoxelShape'in X, Y ve Z eksenlerindeki ilk ve son dolu koordinat düzlemlerini `min(axis)` ve `max(axis)` ile sorgulamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:29-37` ile `world/phys/shapes/Shapes.java:44-120` incelendi. `MathOracle`, altı axis sorgusunu doğrudan Java API'sine verir; beklenen endpoint'leri test içinde yeniden hesaplamaz.

Test girdisi önce `Shapes.or(first, tail...)` ile bir shape oluşturur, ardından aynı shape üzerinde üç eksenin min ve max sorgularını yapar. Sonuçlar ham f64 bit string'leri olarak taşınır.

## Davranış sözleşmesi

- Dolu shape için `min(axis)`, seçilen eksendeki ilk dolu hücrenin başlangıç koordinatını döndürür.
- Dolu shape için `max(axis)`, seçilen eksendeki son dolu hücrenin bitiş koordinatını döndürür.
- Empty shape için `min(axis)` pozitif sonsuz, `max(axis)` negatif sonsuz döndürür.
- X, Y ve Z sonuçları Java double ham bitleriyle karşılaştırılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | Epsilon altında genişliği bulunan ve Java'da empty olan shape |
| disjoint | 1 | Ayrık iki box ile farklı eksen sınırlarını gözlemleyen birleşim |
| negative | 1 | Negatif ve pozitif koordinatları birlikte taşıyan birleşim |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "axis_limits"` ve `shapes` dizisini kullanır. Sonuç `min_x`, `max_x`, `min_y`, `max_y`, `min_z` ve `max_z` alanlarında ham f64 bit string'leri taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliğiyle doğrulandı.

Math debug/release raporları `compare-druvbg3h` ve `compare-k2ed8y70`; birleşik debug/release raporları `compare-h1kbcaxz` ve `compare-js7_mas4` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
