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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `a68619cf6f1338df37784d434e06d0952b6dbaab0962ece1f5a77fcbda2b03a6` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `b0cbcb74aa0f40450d03ba7ed23e6155f13776414b6dc51e556a0b02e08fa2c4` kimliğiyle doğrulandı.

Math debug/release raporları `compare-qjy5sea_` ve `compare-hv0y0sp6`; birleşik debug/release raporları `compare-be91aw33` ve `compare-h5dd6zge` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
