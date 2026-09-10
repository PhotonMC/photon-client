# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1398 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1398/1398** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug) ve `compare-wrj9a6ex` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
