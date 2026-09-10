# P01.d — Shapes.rotateAttachFace doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573.

Birleşik referans kimliği: ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573.

Yerel raporlar: compare-hiau72j0 (math debug), compare-bttpme0o (math release), compare-k17hah5m (birleşik debug) ve compare-hiiz6u35 (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
