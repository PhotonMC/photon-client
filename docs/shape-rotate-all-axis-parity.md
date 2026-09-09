# P01.d — Shapes.rotateAllAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis` yolunu X, Y ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-all-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 580 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **580/580** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, üç dönüşün sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1034/1034** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8db71332cfbcc696bf89823167fb052b36cf4e81b874256da28436326d9cdadc`.

Birleşik referans kimliği: `d912c38089b8e4b8d1bbab5d2d9ad532a60108e293727c14cc25d6f3ad0e7333`.

Yerel raporlar: `compare-cgdwsv4p` (math debug), `compare-ydb0q8z5` (math release), `compare-guo5917x` (birleşik debug) ve `compare-jlxs6tma` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli üç eksenli map ve üç box girdisiyle sınırlıdır. Özel merkez overload ayrı [all-axis merkez parity belgesinde](shape-rotate-all-axis-center-parity.md) doğrulanır. Merkez parametreli diğer overload'lar, `rotateHorizontal`, `rotateAll` yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
