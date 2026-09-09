# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 549 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **549/549** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1003/1003** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f205ceba714d4702c836571f170f9dfb4fd0000df6c1d6a58b646deb7d6c4426`.

Birleşik referans kimliği: `087a802cf2b50f4fd723cf119b31303a75b2a07d7e98062d8b40b0fa9394d3af`.

Yerel raporlar: `compare-2c90a5am` (math debug), `compare-4j73hu_g` (math release), `compare-xm1dqtf3` (birleşik debug) ve `compare-1fuiwzut` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. `initial` rotation, merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
