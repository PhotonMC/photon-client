# P01.d — Shapes.rotateAll referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-parity.md)). Kapsam, north shape'inden altı yön için shape map'i üreten Java yardımcı metodunun varsayılan overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:427-452` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateAll` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotateAll(north)` overload'ını kapsar. Özel merkez overload'ı ayrı [sözleşme](shape-rotate-all-center-contract.md) ve [parity](shape-rotate-all-center-parity.md) belgelerinde ele alınır. `initial` ve `rotationCenter` alanlarını birlikte alan overload ayrı [sözleşme](shape-rotate-all-initial-contract.md) ve [parity](shape-rotate-all-initial-parity.md) belgelerinde ele alınır. `rotateHorizontal` ve `rotateAttachFace` yön map'leri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAll(north)` varsayılan olarak `IDENTITY` başlangıç dönüşü ve blok merkezi `(0.5, 0.5, 0.5)` kullanır.
- NORTH anahtarındaki değer özgün shape'tir; EAST, SOUTH, WEST, UP ve DOWN anahtarları sırasıyla `BLOCK_ROT_Y_90`, `BLOCK_ROT_Y_180`, `BLOCK_ROT_Y_270`, `BLOCK_ROT_X_270` ve `BLOCK_ROT_X_90` dönüşleriyle üretilir.
- Java `EnumMap` direction enum sırasını korur; sonuç JSONL'de DOWN, UP, NORTH, SOUTH, WEST, EAST olarak gözlenir. Empty shape altı anahtarda da empty snapshot üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Altı dönüşü ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | `1e-7` eşiğinin altında aralık |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_all"` kullanır. Sonuç `shapes` dizisinde `direction` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `7d929c43d190008e7958b13c5cd6ed1ecfd727fb2477efae2e307b70d7888deb` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `79c0a319fee88ab66551f0d6e482274fbf9e528c52ae097bb43853784b4e846a` kimliğiyle doğrulandı.

Math debug/release raporları `compare-yuxqj_ix` ve `compare-fpos2gkp`; birleşik debug/release raporları `compare-witt8p3m` ve `compare-pabdz7bu` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
