# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 574 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **574/574** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG ve ROT_180_FACE_XZ başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1028/1028** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 1431e2c083406bf997d7d068455e909ece993da6c2960b5a58aeb3505a1bbecf.

Birleşik referans kimliği: 973124eeb6072de7c2a8f42e33b01304c2a5e79e2ef0a87968d085bb7084a0ee.

Yerel raporlar: compare-f5pqb8w9 (math debug), compare-1ghjhwjn (math release), compare-4pe6qeij (birleşik debug) ve compare-94buxmhn (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
