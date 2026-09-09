# P01.d — Shapes.rotateAll initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, initial, rotationCenter)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 577 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **577/577** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla DOWN, UP, NORTH, SOUTH, WEST ve EAST yönlerinin enum sırası, bileşik dönüşleri ve özel merkez davranışı karşılaştırıldı.
- Birleşik altı aile corpus'u **1031/1031** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5df31dedf41786bcd5bb38fc117404df6a83da03bce464c1a637063e5b3b5cbb`.

Birleşik referans kimliği: `cc0f402235532cc834549c53fd069f064a8e771ef6ba6e2943e472b1d9b6b06d`.

Yerel raporlar: `compare-jrhf0iba` (math debug), `compare-8berq09t` (math release), `compare-rpvlapca` (birleşik debug) ve `compare-dwzkalq0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri, üç sentetik box senaryosunu ve verilen `rotationCenter` değerlerini kapsar. `NORTH` yolunun blok merkezini koruması ayrıca bu sözleşmede sabitlenmiştir. Rust `ShapeRotation` enum'undaki diğer başlangıç değerleri, Java'nın kalan `OctahedralGroup` değerleri ve diğer `rotateAll` overload'ları ayrı dilimlerdir.
