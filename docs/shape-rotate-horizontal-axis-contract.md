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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliğiyle doğrulandı.

Math debug/release raporları `compare-y28xdodh` ve `compare-_7cfq3in`; birleşik debug/release raporları `compare-cxv1an3r` ve `compare-1jjs0aun` dizinlerindedir. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
