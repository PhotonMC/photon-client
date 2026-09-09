# P01.d — Shapes.rotateAttachFace doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1301 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1301/1301** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935.

Birleşik referans kimliği: e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935.

Yerel raporlar: compare-zq98bs3k (math debug), compare-zgg863t0 (math release), compare-yz31bmcf (birleşik debug) ve compare-90dzj4qp (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
