# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs altı JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406.

Birleşik referans kimliği: 53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406.

Yerel raporlar: compare-_lwapdio (math debug), compare-ed7p25g6 (math release), compare-uh_quimp (birleşik debug) ve compare-233_yr3a (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı seçili initial değeri ve altı sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
