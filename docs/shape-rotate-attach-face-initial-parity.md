# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 983 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **983/983** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671`.

Birleşik referans kimliği: `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671`.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug) ve `compare-09rh6rbh` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
