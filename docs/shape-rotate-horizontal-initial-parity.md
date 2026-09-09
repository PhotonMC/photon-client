# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs altı JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 616 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **616/616** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1070/1070** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 72703f27997a3fbdb61c87ead93863a7a53806ebb3533f7e4c676f9a81591df9.

Birleşik referans kimliği: e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14.

Yerel raporlar: compare-8rckp6vr (math debug), compare-4eof6x0t (math release), compare-s3ki0jz5 (birleşik debug) ve compare-fng7itjt (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı seçili initial değeri ve altı sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
