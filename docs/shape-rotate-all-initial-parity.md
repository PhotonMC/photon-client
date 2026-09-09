# P01.d — Shapes.rotateAll initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north, initial, rotationCenter)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 989 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **989/989** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla DOWN, UP, NORTH, SOUTH, WEST ve EAST yönlerinin enum sırası, bileşik dönüşleri ve özel merkez davranışı karşılaştırıldı.
- Birleşik altı aile corpus'u **1491/1491** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `0f630454331682d5ce34027e78b67ab2555fdb80eaa4f4e4bb2485885abebab6`.

Birleşik referans kimliği: `0f630454331682d5ce34027e78b67ab2555fdb80eaa4f4e4bb2485885abebab6`.

Yerel raporlar: `compare-jccgb79g` (math debug), `compare-r5e2j3zv` (math release), `compare-n67o4lpe` (birleşik debug) ve `compare-y4v3jm1g` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri, üç sentetik box senaryosunu ve verilen `rotationCenter` değerlerini kapsar. `NORTH` yolunun blok merkezini koruması ayrıca bu sözleşmede sabitlenmiştir. Rust `ShapeRotation` enum'undaki diğer başlangıç değerleri, Java'nın kalan `OctahedralGroup` değerleri ve diğer `rotateAll` overload'ları ayrı dilimlerdir.
