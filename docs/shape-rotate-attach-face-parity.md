# P01.d — Shapes.rotateAttachFace doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1506 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1506/1506** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57.

Birleşik referans kimliği: 8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57.

Yerel raporlar: compare-wn40y3iw (math debug), compare-9phkvs2k (math release), compare-a415_00_ (birleşik debug) ve compare-djb85w4w (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
