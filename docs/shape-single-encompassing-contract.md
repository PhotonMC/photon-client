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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `a96f6ac711181f55389712d207cddb3ac84e94a0b65b0aa592874d653bb8d9fd` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `b23e0bd7aa5f9df5baa9862e79e09f08f309c7b06b36028da91d501f3ddae578` kimliğiyle doğrulandı.

Math debug/release raporları `compare-fu6yzw05` ve `compare-jp_d2q8j`; birleşik debug/release raporları `compare-ixe0azli` ve `compare-k8ldje4k` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
