# P01.d — Shapes.rotateHorizontal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontal(north)` yolunu NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 640 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **640/640** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri dört yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1094/1094** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `81717c09b76bd67677a312941d9eac7dd5776ef9563213a1ab5062f8ef1e32a7`.

Birleşik referans kimliği: `81717c09b76bd67677a312941d9eac7dd5776ef9563213a1ab5062f8ef1e32a7`.

Yerel raporlar: `compare-52kzwns3` (math debug), `compare-8rgxxwte` (math release), `compare-tmxer50a` (birleşik debug) ve `compare-t62jxkbf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli dört yön map'iyle sınırlıdır. Desteklenen initial overload ayrı [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md); merkez parametreli overload, `rotateAll`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
