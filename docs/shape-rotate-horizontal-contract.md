# P01.d — Shapes.rotateHorizontal referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-parity.md)). Kapsam, north shape'inden dört yatay yön için shape map'i üreten Java yardımcı metodunun varsayılan overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:404-425` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateHorizontal` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotateHorizontal(north)` overload'ını kapsar. `initial` rotation, özel merkez ve `rotateAll`/`rotateAttachFace` yön map'leri ayrı dilimlerdir.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `403f9d74fdc17371a3bb9f180e0f860010c3509bb0142714d50914e30a0963dc` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c` kimliğiyle doğrulandı.

Math debug/release raporları `compare-e7w7szf1` ve `compare-cp2yxn59`; birleşik debug/release raporları `compare-sq3wafy1` ve `compare-1i1dj3ib` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
