# P01.d — Shapes.rotateAttachFace doğrulaması

src/shape.rs, Java 26.2 Shapes.rotateAttachFace(north) yolunu floor, wall ve ceiling map'leriyle uygular; floor ve ceiling başlangıçları Java OctahedralGroup.compose sırasıyla bileştirir, iç map'leri NORTH/SOUTH/WEST/EAST sırasına taşır. examples/behavior_adapter.rs üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 923 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **923/923** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri üç attach face'i, her iç yön map'ini, bileşik dönüş yönünü ve enum sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1425/1425** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: 46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422.

Birleşik referans kimliği: 46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422.

Yerel raporlar: compare-85j8zrrj (math debug), compare-eq8g_l1t (math release), compare-ufi9ykko (birleşik debug) ve compare-fuqao9g1 (birleşik release) dizinlerinde, .data/runs/behavior/ altında bulunur.

## Sınırlar

Bu kanıt yalnızca identity başlangıçlı tek argümanlı overload, üç sentetik box ve floor/wall/ceiling içindeki dört yatay yön map'iyle sınırlıdır. initial rotation overload'ı, özel merkezli yön map'leri ve diğer OctahedralGroup değerleri ayrı dilimlerdir.
