# P01.d — Shapes.rotateAllAxis referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-axis-parity.md)). Kapsam, bir north shape'inden üç eksen yönü için shape map'i üreten Java yardımcı metodu içerir. `Shapes.rotateHorizontal` çıktısı ayrı [sözleşme](shape-rotate-horizontal-contract.md) ve [parity](shape-rotate-horizontal-parity.md) belgelerinde; `Shapes.rotateAll` çıktısı ayrı [sözleşme](shape-rotate-all-contract.md) ve [parity](shape-rotate-all-parity.md) belgelerinde ele alınır. Özel merkez overload'ı ayrı [sözleşme](shape-rotate-all-axis-center-contract.md) ve [parity](shape-rotate-all-axis-center-parity.md) belgelerinde ele alınır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:387-402` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. Varsayılan ve merkez overload aynı `Shapes.rotate` signed eksen yolunu kullanır. `MathOracle` doğrudan `Shapes.rotateAllAxis` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz. Bu dilim yalnızca `rotateAllAxis(north)` overload'ını kapsar; özel merkez overload'ı ayrı [sözleşme](shape-rotate-all-axis-center-contract.md) ve [parity](shape-rotate-all-axis-center-parity.md) belgelerinde doğrulanır.

## Davranış sözleşmesi

- `Shapes.rotateAllAxis(north)` üç girişli bir `EnumMap` döndürür: X anahtarındaki değer `north` shape'inin `BLOCK_ROT_Y_90` dönüşümü, Y anahtarındaki değer `BLOCK_ROT_X_90` dönüşümü, Z anahtarındaki değer ise verilen shape'tir.
- Varsayılan dönüş merkezi `(0.5, 0.5, 0.5)` değeridir ve iki dönüşlü varyant `Shapes.rotate` ile aynı signed eksen, hücre ve koordinat kurallarını kullanır.
- Java `EnumMap` eksen enum sırasını korur; sonuç JSONL'de X, Y, Z olarak gözlenir. Empty shape her üç anahtarda da empty snapshot üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Üç dönüşü ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | `1e-7` eşiğinin altında aralık |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all_axis"` kullanır. Sonuç `shapes` dizisinde `axis` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `6c664e0096c0ab12467c89aed7a906a49cbc22a2eb8b747dadf039bdd64c5f34` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `a3f0878cf89549c06659e43bb88142eced597ada4cc7f8e7ceb2c84d2a0ef851` kimliğiyle doğrulandı.

Math debug/release raporları `compare-vw9w8ekf` ve `compare-p44xr16i`; birleşik debug/release raporları `compare-xj3gnj4z` ve `compare-u91oyin8` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
