# P01.d — Shapes.rotateAttachFace doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 630 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **630/630** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1084/1084** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc.

Birleşik referans kimliği: ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc.

Yerel raporlar: compare-vn_8nl_6 (math debug), compare-e350pb7p (math release), compare-bdb0mgf_ (birleşik debug) ve compare-0a9n4nfd (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
