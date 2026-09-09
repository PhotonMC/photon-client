# P01.d — VoxelShape.singleEncompassing referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-single-encompassing-parity.md)). Kapsam, bir VoxelShape'in boşsa boş shape'e, doluysa bütün dolu hücrelerini içine alan tek box shape'e indirgenmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:29-65` ile `world/phys/shapes/Shapes.java:44-120` incelendi. `MathOracle`, `singleEncompassing()` çağrısını doğrudan yapar; beklenen kutu sınırları test içinde yeniden hesaplanmaz.

Test girdisi önce `Shapes.or(first, tail...)` ile bir shape oluşturur, ardından bu shape üzerinde `singleEncompassing()` çağrılır. Bu kurulum, yöntemin tek kutu, temas eden kutu ve arada boşluk bulunan kutu birleşimlerindeki davranışını aynı snapshot ile gözlemlemeyi sağlar.

## Davranış sözleşmesi

- Shape boşsa sonuç Java'nın canonical empty shape'idir; kutu ve bounds alanı bulunmaz.
- Dolu shape için X, Y ve Z eksenlerinde ilk ve son dolu koordinat düzlemleri seçilir.
- Bu altı sınırla `Shapes.box` çağrılır ve aradaki boş hücreler de kapsama kutusuna alınır.
- Non-empty sonuç tek bir box snapshot'ı ve aynı sınırları taşıyan bounds alanı üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | Epsilon altında genişliği bulunan ve Java'da empty olan shape |
| touching | 1 | Yüzeyden temas eden iki box'ın birleşimi |
| disjoint | 1 | Arasında boşluk bulunan ve farklı eksen aralıklarına sahip iki box'ın birleşimi |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "single_encompassing"` ve `shapes` dizisini kullanır. Sonuç `shape` alanında shape snapshot'ı taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `22a5e87b3b46a0afade4d43a8a9e0dfaf96d8d892ea18ddc0ba5ebf77750772c` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `22a5e87b3b46a0afade4d43a8a9e0dfaf96d8d892ea18ddc0ba5ebf77750772c` kimliğiyle doğrulandı.

Math debug/release raporları `compare-p4m_t_r8` ve `compare-eskrxypz`; birleşik debug/release raporları `compare-skwbedqw` ve `compare-9bwuf8cs` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
