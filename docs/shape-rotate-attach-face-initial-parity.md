# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 622 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **622/622** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1076/1076** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `fbb737e6925fc93a52c2d06bcffcf4d171ed6e38c6f80aabbb3f9ab908568e5e`.

Birleşik referans kimliği: `fbb737e6925fc93a52c2d06bcffcf4d171ed6e38c6f80aabbb3f9ab908568e5e`.

Yerel raporlar: `compare-d3woeteb` (math debug), `compare-o4xpn51x` (math release), `compare-5ww6qivx` (birleşik debug) ve `compare-gzcepmlm` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
