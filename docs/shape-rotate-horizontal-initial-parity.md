# P01.d — Shapes.rotateHorizontal initial overload doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateHorizontal(north, initial) yolunun Rust ShapeRotation alt kümesini Java compose sırasıyla uygular; examples/behavior_adapter.rs altı JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 601 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **601/601** math senaryosunda eşleşti.
- IDENTITY, ROT_90_X_NEG, ROT_180_FACE_XZ, ROT_90_Y_NEG, ROT_90_Z_NEG ve ROT_180_FACE_XY başlangıçlarıyla dört yatay yönün enum sırası, koordinat snapshot'ları ve bileşik dönüşleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1055/1055** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3.

Birleşik referans kimliği: 716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52.

Yerel raporlar: compare-8dsunddp (math debug), compare-zvv3a_3_ (math release), compare-9k3hskia (birleşik debug) ve compare-yw9njwwe (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı seçili initial değeri ve altı sentetik box senaryosunu kapsar. Rust ShapeRotation enum'undaki diğer başlangıç değerleri, Java'nın kalan OctahedralGroup değerleri ve özel rotation center overload'ı ayrı dilimlerdir.
