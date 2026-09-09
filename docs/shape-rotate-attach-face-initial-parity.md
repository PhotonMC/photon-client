# P01.d — Shapes.rotateAttachFace initial overload doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAttachFace(north, initial)` yolunun Rust `ShapeRotation` alt kümesini Java compose sırasıyla uygular; `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 941 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **941/941** math senaryosunda eşleşti.
- `IDENTITY`, `ROT_90_X_NEG` ve `ROT_180_FACE_XZ` başlangıçlarıyla FLOOR, WALL ve CEILING dış sırası ile iç dört yön map'lerinin bileşimleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1443/1443** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6`.

Birleşik referans kimliği: `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6`.

Yerel raporlar: `compare-9rd6f2k4` (math debug), `compare-azcslb8q` (math release), `compare-9epmwbb2` (birleşik debug) ve `compare-vdfamuam` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Diğer `OctahedralGroup` değerleri ve attach-face dışındaki overload'lar ayrı dilimlerdir.
