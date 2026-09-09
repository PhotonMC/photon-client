# P01.d — Shapes.rotateAttachFace initial overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-attach-face-initial-parity.md)). Kapsam, north shape'inden floor, wall ve ceiling attach face'leri için dört yatay yön map'i üreten `Shapes.rotateAttachFace(north, initial)` overload'ının Rust `ShapeRotation` alt kümesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:454-466`, `com/mojang/math/OctahedralGroup.java` ve `world/level/block/state/properties/AttachFace.java:5-8` incelendi. `MathOracle` doğrudan initial overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla gözlenen yolları kapsar. Rust `ShapeRotation` enum'undaki diğer başlangıçlar ve Java'nın kalan `OctahedralGroup` değerleri ayrı dilimlerdir.

## Davranış sözleşmesi

- `Shapes.rotateAttachFace(north, initial)`, `FLOOR`, `WALL` ve `CEILING` anahtarlarıyla her biri dört yatay yön içeren bir map üretir.
- `WALL` map'i initial dönüşünü kullanır. `FLOOR`, `BLOCK_ROT_X_270.compose(initial)`; `CEILING` ise `BLOCK_ROT_Y_180.compose(BLOCK_ROT_X_90).compose(initial)` başlangıcıyla `rotateHorizontal` yolunu kullanır.
- Dış map sırası Java `AttachFace.values()` ile `FLOOR`, `WALL`, `CEILING`; iç map sırası Java `Direction` enum ile `NORTH`, `SOUTH`, `WEST`, `EAST` olarak gözlenir.
- Her iç map'in yatay yön bileşikleri Java `OctahedralGroup.compose` sırasını korur; shape koordinatları ve dolu hücre indisleri signed eksen dönüşümleriyle taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| identity | 1 | Asimetrik box ve `IDENTITY` initial |
| x-90 | 1 | Asimetrik box ve `ROT_90_X_NEG` initial |
| y-180 | 1 | Y ekseni yarım cube-grid box ve `ROT_180_FACE_XZ` initial |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "rotate_attach_face_initial"`, `box` ve `initial` alanlarını kullanır. Sonuç `faces` dizisinde attach face adı ile iç yön/shape snapshot dizisini taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52` kimliğiyle doğrulandı.

Math debug/release raporları `compare-8dsunddp` ve `compare-zvv3a_3_`; birleşik debug/release raporları `compare-9k3hskia` ve `compare-yw9njwwe` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
