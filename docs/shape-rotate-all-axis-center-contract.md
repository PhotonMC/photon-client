# P01.d — Shapes.rotateAllAxis merkez overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-axis-center-parity.md)). Kapsam, north shape'inden X, Y ve Z eksen map'i üreten `Shapes.rotateAllAxis(north, rotationCenter)` overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:387-402`, `Shapes.java:323-372` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan merkez overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca özel merkez alan iki argümanlı overload'ı kapsar. Varsayılan `rotateAllAxis(north)`, `rotateHorizontalAxis`, yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAllAxis(north, rotationCenter)`, X, Y ve Z anahtarlarıyla bir `EnumMap` üretir.
- X anahtarındaki değer `north` shape'inin `BLOCK_ROT_Y_90` dönüşümünü, Y anahtarındaki değer `BLOCK_ROT_X_90` dönüşümünü verilen merkezle çağırır; Z anahtarındaki değer özgün shape'tir ve merkezi kullanmaz.
- Java `EnumMap` eksen sırasını korur; sonuç JSONL'de X, Y ve Z olarak gözlenir. Empty shape üç anahtarda da empty snapshot üretir.
- X ve Y dönüşleri signed eksen permütasyonu, hücre indisleri ve merkez bileşenlerinin output eksenlerine taşınması bakımından doğrudan `Shapes.rotate` sözleşmesini izler.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Üç eksen dönüşünü ayırt eden box ve farklı merkez bileşenleri |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box ve özel merkez |
| empty shape | 1 | Eşik altı aralık ve özel merkez |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all_axis_center"`, `box` ve `center` alanlarını kullanır. Sonuç `shapes` dizisinde `axis` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliğiyle doğrulandı.

Math debug/release raporları `compare-druvbg3h` ve `compare-k2ed8y70`; birleşik debug/release raporları `compare-h1kbcaxz` ve `compare-js7_mas4` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
