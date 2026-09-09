# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 670 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **670/670** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1124/1124** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7463d45cd07a5207a5d29e6be51ea489c541efadb966932e595b98fdda8d1940`.

Birleşik referans kimliği: `7463d45cd07a5207a5d29e6be51ea489c541efadb966932e595b98fdda8d1940`.

Yerel raporlar: `compare-ra03eisz` (math debug), `compare-up24hdi8` (math release), `compare-rgbylm8r` (birleşik debug) ve `compare-445klgjk` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
