# P01.d — Shapes.rotateHorizontalAxis merkez overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-axis-center-parity.md)). Kapsam, Z ekseni shape'inden X ve Z eksen map'i üreten `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)` overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:379-381`, `Shapes.java:323-372` ve `com/mojang/math/OctahedralGroup.java` içindeki `BLOCK_ROT_Y_90` alias'ı incelendi. `MathOracle` doğrudan merkez overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca özel merkez alan iki argümanlı overload'ı kapsar. Varsayılan `rotateHorizontalAxis(zAxis)`, `rotateAllAxis` ve diğer yön map'leri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateHorizontalAxis(zAxis, rotationCenter)`, `IDENTITY` merkez yoluyla bir X/Z `EnumMap` üretir.
- X anahtarındaki değer `zAxis` shape'inin `BLOCK_ROT_Y_90` dönüşümünü verilen merkezle çağırır; Z anahtarındaki değer özgün shape'tir ve merkezi kullanmaz.
- Java `EnumMap` eksen sırasını korur; sonuç JSONL'de X ve Z olarak gözlenir. Empty shape iki anahtarda da empty snapshot üretir.
- X dönüşü signed eksen permütasyonu, hücre indisleri ve merkez bileşenlerinin output eksenlerine taşınması bakımından doğrudan `Shapes.rotate` sözleşmesini izler.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Yatay olmayan koordinatları ayırt eden box ve farklı merkez bileşenleri |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box ve özel merkez |
| empty shape | 1 | Eşik altı aralık ve özel merkez |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_horizontal_axis_center"`, `box` ve `center` alanlarını kullanır. Sonuç `shapes` dizisinde `axis` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `c121387466c02d22c597319b4f93bfab63c1d5b60513759b3db27ed352523c07` kimliğiyle doğrulandı.

Math debug/release raporları `compare-ihodhcd4` ve `compare-1euc1ro8`; birleşik debug/release raporları `compare-_c9tjida` ve `compare-jbm02l6z` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
