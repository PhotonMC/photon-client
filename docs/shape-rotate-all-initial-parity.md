# P01.d — Shapes.rotateAll initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, initial, rotationCenter)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1478 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1478/1478** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla DOWN, UP, NORTH, SOUTH, WEST ve EAST yönlerinin enum sırası, bileşik dönüşleri ve özel merkez davranışı karşılaştırıldı.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug) ve `compare-cvif6gdr` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri, üç sentetik box senaryosunu ve verilen `rotationCenter` değerlerini kapsar. `NORTH` yolunun blok merkezini koruması ayrıca bu sözleşmede sabitlenmiştir. Rust `ShapeRotation` enum'undaki diğer başlangıç değerleri, Java'nın kalan `OctahedralGroup` değerleri ve diğer `rotateAll` overload'ları ayrı dilimlerdir.
