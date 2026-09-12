# P01.d — Shapes.rotateHorizontal initial overload referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, north shape'inden dört yatay yön map'i üreten Shapes.rotateHorizontal(north, initial) overload'ının Rust ShapeRotation alt kümesidir.

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

Bu dilimdeki senaryoların bulunduğu güncel math referansı 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b kimliğiyle doğrulandı.

Math debug/release raporları compare-00tgiqc4 ve compare-aylr4ozm; birleşik debug/release raporları compare-5bjkz31s ve compare-zfkba6ts dizinlerindedir, .data/runs/behavior/ altında bulunur. Ham girdiler ve Java çıktıları yalnızca .data/ altında tutulur.


## Client-parity doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs altı JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b.

Birleşik referans kimliği: 8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b.

Yerel raporlar: compare-00tgiqc4 (math debug), compare-aylr4ozm (math release), compare-5bjkz31s (birleşik debug) ve compare-zfkba6ts (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı seçili initial değeri ve altı sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
