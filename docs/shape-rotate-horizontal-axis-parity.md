# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-horizontal-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` varsayılan üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 571 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **571/571** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1025/1025** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `bf8eac3fe88ccc7eac0052e8b44cb5dab3dfb456982cd6a700ad1f08fadc02a2`.

Birleşik referans kimliği: `3133f0b82eddafd4383cd19cad3d4e53db2b9051614cfc78968ee4d6d98810b8`.

Yerel raporlar: `compare-wbo29fbq` (math debug), `compare-7m5csdkl` (math release), `compare-tyoie69t` (birleşik debug) ve `compare-_lv2ppe_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload ayrı [yatay eksen merkez parity belgesinde](shape-rotate-horizontal-axis-center-parity.md) doğrulanır; `rotateAllAxis` sonucu ayrı [all-axis parity belgesinde](shape-rotate-all-axis-parity.md) doğrulanır; yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
