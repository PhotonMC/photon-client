# P01.d — Shapes.rotateHorizontal initial overload referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-rotate-horizontal-initial-parity.md)). Kapsam, north shape'inden dört yatay yön map'i üreten Shapes.rotateHorizontal(north, initial) overload'ının Rust ShapeRotation alt kümesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. world/phys/shapes/Shapes.java:404-425, com/mojang/math/OctahedralGroup.java ve SymmetricGroup3.java incelendi. MathOracle doğrudan initial overload'ını çağırır; map sonuçları oracle içinde yeniden hesaplanmaz.

Bu dilim Java overload'ının IDENTITY, ROT_90_X_NEG ve ROT_180_FACE_XZ başlangıçlarıyla gözlenen yollarını kapsar. Rust ShapeRotation enum'unda bulunan diğer başlangıçlar ve Java'nın kalan OctahedralGroup değerleri ayrı dilimlerdir. Özel rotation center overload'ı da ayrı kapsamdır.

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

Girdiler family: "math", operation: "shape", kind: "rotate_horizontal_initial", box ve initial alanlarını kullanır. Sonuç shapes dizisinde Direction adı ve shape snapshot çiftlerini taşır.

## Java yakalama

Depo kökünden:

Yakalama komutu: python3 tools/behavior/run.py capture --family math

Bu dilimdeki senaryoların bulunduğu güncel math referansı 5df31dedf41786bcd5bb38fc117404df6a83da03bce464c1a637063e5b3b5cbb kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı cc0f402235532cc834549c53fd069f064a8e771ef6ba6e2943e472b1d9b6b06d kimliğiyle doğrulandı.

Math debug/release raporları compare-jrhf0iba ve compare-8berq09t; birleşik debug/release raporları compare-rpvlapca ve compare-dwzkalq0 dizinlerindedir, .data/runs/behavior/ altında bulunur. Ham girdiler ve Java çıktıları yalnızca .data/ altında tutulur.
