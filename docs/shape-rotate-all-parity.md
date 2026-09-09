# P01.d — Shapes.rotateAll doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar. Özel merkez overload sonucu ayrı [parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload sonucu ayrı [initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406`.

Birleşik referans kimliği: `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406`.

Yerel raporlar: `compare-_lwapdio` (math debug), `compare-ed7p25g6` (math release), `compare-uh_quimp` (birleşik debug) ve `compare-233_yr3a` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli altı yön map'iyle sınırlıdır. Özel merkez overload ayrı [rotateAll merkez parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload ayrı [rotateAll initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır; `rotateHorizontal`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
