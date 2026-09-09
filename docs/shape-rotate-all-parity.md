# P01.d — Shapes.rotateAll doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar. Özel merkez overload sonucu ayrı [parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload sonucu ayrı [initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 616 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **616/616** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1070/1070** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `72703f27997a3fbdb61c87ead93863a7a53806ebb3533f7e4c676f9a81591df9`.

Birleşik referans kimliği: `e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14`.

Yerel raporlar: `compare-8rckp6vr` (math debug), `compare-4eof6x0t` (math release), `compare-s3ki0jz5` (birleşik debug) ve `compare-fng7itjt` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli altı yön map'iyle sınırlıdır. Özel merkez overload ayrı [rotateAll merkez parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload ayrı [rotateAll initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır; `rotateHorizontal`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
