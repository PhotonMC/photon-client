# P01.d — Shapes.rotateAll doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar. Özel merkez overload sonucu ayrı [parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload sonucu ayrı [initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 601 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **601/601** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1055/1055** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3`.

Birleşik referans kimliği: `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52`.

Yerel raporlar: `compare-8dsunddp` (math debug), `compare-zvv3a_3_` (math release), `compare-9k3hskia` (birleşik debug) ve `compare-yw9njwwe` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli altı yön map'iyle sınırlıdır. Özel merkez overload ayrı [rotateAll merkez parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload ayrı [rotateAll initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır; `rotateHorizontal`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
