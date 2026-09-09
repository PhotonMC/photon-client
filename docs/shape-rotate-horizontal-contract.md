# P01.d — Shapes.rotateHorizontal referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-parity.md)). Kapsam, north shape'inden dört yatay yön için shape map'i üreten Java yardımcı metodunun varsayılan overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:404-425` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateHorizontal` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotateHorizontal(north)` overload'ını kapsar. Desteklenen initial overload ayrı [sözleşme](shape-rotate-horizontal-initial-contract.md) ve [parity](shape-rotate-horizontal-initial-parity.md) belgelerinde; özel merkez ve `rotateAll`/`rotateAttachFace` yön map'leri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateHorizontal(north)` varsayılan olarak `IDENTITY` başlangıç dönüşü ve blok merkezi `(0.5, 0.5, 0.5)` kullanır.
- NORTH anahtarındaki değer özgün shape'tir; EAST, SOUTH ve WEST anahtarları sırasıyla `BLOCK_ROT_Y_90`, `BLOCK_ROT_Y_180` ve `BLOCK_ROT_Y_270` dönüşleriyle üretilir.
- Java `EnumMap` direction enum sırasını korur; yalnızca yatay anahtarlar bulunduğu için sonuç JSONL'de NORTH, SOUTH, WEST, EAST olarak gözlenir. Empty shape dört anahtarda da empty snapshot üretir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| asimetrik shape | 1 | Dört yatay dönüşü ayırt eden box |
| grid boyutu | 1 | Y ekseni yarım olan cube-grid box |
| empty shape | 1 | `1e-7` eşiğinin altında aralık |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_horizontal"` kullanır. Sonuç `shapes` dizisinde `direction` ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `7d929c43d190008e7958b13c5cd6ed1ecfd727fb2477efae2e307b70d7888deb` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `79c0a319fee88ab66551f0d6e482274fbf9e528c52ae097bb43853784b4e846a` kimliğiyle doğrulandı.

Math debug/release raporları `compare-yuxqj_ix` ve `compare-fpos2gkp`; birleşik debug/release raporları `compare-witt8p3m` ve `compare-pabdz7bu` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
