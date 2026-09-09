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

Bu dilimdeki senaryoların bulunduğu güncel math referansı `88893f7d2054eb1f0f13242d6d887e1ae0d0e1dcb1a99c08a190cff768f8b841` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `fa080fc05d94f4eec8044ce6e4989a288e586b7d5065e1926ba18b75170a3bdb` kimliğiyle doğrulandı.

Math debug/release raporları `compare-p21yz9ko` ve `compare-tvqo_c61`; birleşik debug/release raporları `compare-fgeq_2hx` ve `compare-b0c437h_` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
