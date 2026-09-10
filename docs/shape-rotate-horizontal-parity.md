# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1783 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1783/1783** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Yerel raporlar: `compare-1g27oorn` (math debug), `compare-bkcvw7oi` (math release), `compare-qmmqnzwk` (birleşik debug) ve `compare-ogma6668` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. Desteklenen initial overload ayrı [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md); merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
