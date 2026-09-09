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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `88893f7d2054eb1f0f13242d6d887e1ae0d0e1dcb1a99c08a190cff768f8b841` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `fa080fc05d94f4eec8044ce6e4989a288e586b7d5065e1926ba18b75170a3bdb` kimliğiyle doğrulandı.

Math debug/release raporları `compare-p21yz9ko` ve `compare-tvqo_c61`; birleşik debug/release raporları `compare-fgeq_2hx` ve `compare-b0c437h_` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
