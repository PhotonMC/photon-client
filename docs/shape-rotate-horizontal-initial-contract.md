# P01.d — Shapes.rotateHorizontal initial overload referans sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-initial-parity.md)). Kapsam, north shape'inden dört yatay yön map'i üreten Shapes.rotateHorizontal(north, initial) overload'ının Rust ShapeRotation alt kümesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. world/phys/shapes/Shapes.java:404-425, com/mojang/math/OctahedralGroup.java ve SymmetricGroup3.java incelendi. MathOracle doğrudan initial overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim Java overload'ının IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla gözlenen yollarını kapsar. Rust ShapeRotation enum'unda bulunan diğer başlangıçlar ve Java'nın kalan OctahedralGroup değerleri ayrı dilimlerdir. Özel rotation center overload'ı da ayrı kapsamdır.

## Davranış sözleşmesi

- Shapes.rotateHorizontal(north, initial), NORTH, SOUTH, WEST ve EAST anahtarlarıyla bir EnumMap üretir.
- NORTH değeri initial dönüşünü kullanır. EAST, SOUTH ve WEST değerleri sırasıyla BLOCK_ROT_Y_90, BLOCK_ROT_Y_180 ve BLOCK_ROT_Y_270 dönüşlerinin initial ile Java compose sırasındaki bileşikleridir.
- Java Direction enum sırası sonuç JSONL'ye NORTH, SOUTH, WEST, EAST olarak yansır.
- Dönüşler koordinat düzlemlerini ve dolu hücre indislerini Shapes.rotate sözleşmesiyle aynı signed eksen permütasyonu olarak taşır; başlangıç bileşiğinin sırası output'u belirler.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| identity | 1 | Asimetrik box ve IDENTITY initial |
| x-90 | 1 | Asimetrik box ve ROT_90_X_NEG initial |
| y-180 | 1 | Y ekseni yarım olan cube-grid box ve ROT_180_FACE_XZ initial |
| y-90 | 1 | Asimetrik box ve ROT_90_Y_NEG initial |
| z-90 | 1 | Asimetrik box ve ROT_90_Z_NEG initial |
| xy-180 | 1 | Asimetrik box ve ROT_180_FACE_XY initial |

Girdiler family: "math", operation: "shape", kind: "rotate_horizontal_initial", box ve initial alanlarını kullanır. Sonuç shapes dizisinde Direction adı ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

Yakalama komutu: python3 tools/behavior/run.py capture --family math

Bu dilimdeki senaryoların bulunduğu güncel math referansı ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3 kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı 716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52 kimliğiyle doğrulandı.

Math debug/release raporları compare-8dsunddp ve compare-zvv3a_3_; birleşik debug/release raporları compare-9k3hskia ve compare-yw9njwwe dizinlerindedir, .data/runs/behavior/ altında bulunur. Ham girdiler ve Java çıktıları yalnızca .data/ altında tutulur.
