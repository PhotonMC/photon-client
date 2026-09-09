# P01.d — Shapes.rotateHorizontalAxis referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-axis-parity.md)). Kapsam, Z ekseni için verilen shape'ten yatay X ve Z eksen varyantlarının oluşturulmasını içerir. `Shapes.rotateAllAxis` çıktısı ayrı [sözleşme](shape-rotate-all-axis-contract.md) ve [parity](shape-rotate-all-axis-parity.md) belgelerinde ele alınır. Özel merkez overload'ı ayrı [sözleşme](shape-rotate-horizontal-axis-center-contract.md) ve [parity](shape-rotate-horizontal-axis-center-parity.md) belgelerinde ele alınır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:379-381` ve `com/mojang/math/OctahedralGroup.java` içindeki `BLOCK_ROT_Y_90` alias'ı incelendi. Varsayılan ve merkez overload aynı `Shapes.rotate` signed eksen yolunu kullanır. `MathOracle` doğrudan `Shapes.rotateHorizontalAxis` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

## Davranış sözleşmesi

- `Shapes.rotateHorizontalAxis(zAxis)` iki girişli bir `EnumMap` döndürür: X anahtarındaki değer `zAxis`'in `BLOCK_ROT_Y_90` dönüşümü, Z anahtarındaki değer ise verilen shape'tir.
- Varsayılan dönüş merkezi `(0.5, 0.5, 0.5)` değeridir ve X dönüşü `Shapes.rotate` ile aynı signed eksen, hücre ve koordinat kurallarını kullanır.
- Java `EnumMap` eksen enum sırasını korur; sonuç JSONL'de X sonra Z olarak gözlenir. Empty shape de iki anahtarda empty snapshot üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Yatay olmayan koordinatları ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | `1e-7` eşiğinin altında aralık |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_horizontal_axis"` kullanır. Sonuç `shapes` dizisinde `axis` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `6c664e0096c0ab12467c89aed7a906a49cbc22a2eb8b747dadf039bdd64c5f34` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `a3f0878cf89549c06659e43bb88142eced597ada4cc7f8e7ceb2c84d2a0ef851` kimliğiyle doğrulandı.

Math debug/release raporları `compare-vw9w8ekf` ve `compare-p44xr16i`; birleşik debug/release raporları `compare-xj3gnj4z` ve `compare-u91oyin8` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
