# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1987 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug) ve `compare-jqj2kta_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. Desteklenen initial overload ayrı [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md); merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
