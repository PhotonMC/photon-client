# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs altı JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 630 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **630/630** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1084/1084** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc.

Birleşik referans kimliği: ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc.

Yerel raporlar: compare-vn_8nl_6 (math debug), compare-e350pb7p (math release), compare-bdb0mgf_ (birleşik debug) ve compare-0a9n4nfd (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı seçili initial değeri ve altı sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
