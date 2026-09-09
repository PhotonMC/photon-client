# P01.d — Shapes.rotateAll doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAll(north)` yolunu DOWN, UP, NORTH, SOUTH, WEST ve EAST yön varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar. Özel merkez overload sonucu ayrı [parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload sonucu ayrı [initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 655 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **655/655** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri altı yön anahtarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1109/1109** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Birleşik referans kimliği: `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d`.

Yerel raporlar: `compare-s2mg_zzb` (math debug), `compare-vw7pifam` (math release), `compare-8ia7c6j_` (birleşik debug) ve `compare-fa02lzlz` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan `IDENTITY` başlangıç dönüşü ve blok merkezli altı yön map'iyle sınırlıdır. Özel merkez overload ayrı [rotateAll merkez parity belgesinde](shape-rotate-all-center-parity.md) doğrulanır; başlangıç ve özel merkez overload ayrı [rotateAll initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır; `rotateHorizontal`, `rotateAttachFace` ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
