# P01.d — Shapes.rotateAll referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-all-parity.md)). Kapsam, north shape'inden altı yön için shape map'i üreten Java yardımcı metodunun varsayılan overload'udur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:427-452` ve `com/mojang/math/OctahedralGroup.java` içindeki blok dönüş alias'ları incelendi. `MathOracle` doğrudan `Shapes.rotateAll` çağrısını yapar; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim yalnızca `rotateAll(north)` overload'ını kapsar. `initial` rotation, özel merkez, `rotateHorizontal` ve `rotateAttachFace` yön map'leri ayrı dilimlerdir.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `403f9d74fdc17371a3bb9f180e0f860010c3509bb0142714d50914e30a0963dc` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c` kimliğiyle doğrulandı.

Math debug/release raporları `compare-e7w7szf1` ve `compare-cp2yxn59`; birleşik debug/release raporları `compare-sq3wafy1` ve `compare-1i1dj3ib` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
