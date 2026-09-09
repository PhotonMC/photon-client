# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-horizontal-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` varsayılan üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb`.

Birleşik referans kimliği: `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb`.

Yerel raporlar: `compare-d1bitq7h` (math debug), `compare-ldlepu8u` (math release), `compare-kuzkqdbl` (birleşik debug) ve `compare-n3x2kj9p` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload ayrı [yatay eksen merkez parity belgesinde](shape-rotate-horizontal-axis-center-parity.md) doğrulanır; `rotateAllAxis` sonucu ayrı [all-axis parity belgesinde](shape-rotate-all-axis-parity.md) doğrulanır; yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
