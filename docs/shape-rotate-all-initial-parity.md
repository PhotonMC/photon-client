# P01.d — Shapes.rotateAll initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, initial, rotationCenter)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 601 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **601/601** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla DOWN, UP, NORTH, SOUTH, WEST ve EAST yönlerinin enum sırası, bileşik dönüşleri ve özel merkez davranışı karşılaştırıldı.
- Birleşik altı aile corpus'u **1055/1055** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3`.

Birleşik referans kimliği: `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52`.

Yerel raporlar: `compare-8dsunddp` (math debug), `compare-zvv3a_3_` (math release), `compare-9k3hskia` (birleşik debug) ve `compare-yw9njwwe` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri, üç sentetik box senaryosunu ve verilen `rotationCenter` değerlerini kapsar. `NORTH` yolunun blok merkezini koruması ayrıca bu sözleşmede sabitlenmiştir. Rust `ShapeRotation` enum'undaki diğer başlangıç değerleri, Java'nın kalan `OctahedralGroup` değerleri ve diğer `rotateAll` overload'ları ayrı dilimlerdir.
