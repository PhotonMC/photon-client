# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 619 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **619/619** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1073/1073** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `6c664e0096c0ab12467c89aed7a906a49cbc22a2eb8b747dadf039bdd64c5f34`.

Birleşik referans kimliği: `a3f0878cf89549c06659e43bb88142eced597ada4cc7f8e7ceb2c84d2a0ef851`.

Yerel raporlar: `compare-vw9w8ekf` (math debug), `compare-p44xr16i` (math release), `compare-xj3gnj4z` (birleşik debug) ve `compare-u91oyin8` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
