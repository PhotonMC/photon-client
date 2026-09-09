# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 571 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **571/571** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG ve ROT_180_FACE_XZ başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1025/1025** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: bf8eac3fe88ccc7eac0052e8b44cb5dab3dfb456982cd6a700ad1f08fadc02a2.

Birleşik referans kimliği: 3133f0b82eddafd4383cd19cad3d4e53db2b9051614cfc78968ee4d6d98810b8.

Yerel raporlar: compare-wbo29fbq (math debug), compare-7m5csdkl (math release), compare-tyoie69t (birleşik debug) ve compare-_lv2ppe_ (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca üç seçili initial değeri ve üç sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
