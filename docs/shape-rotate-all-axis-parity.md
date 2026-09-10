# P01.d — Shapes.rotateAllAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis` yolunu X, Y ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-all-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1843 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1843/1843** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, üç dönüşün sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Yerel raporlar: `compare-5_avciij` (math debug), `compare-e6e6agr4` (math release), `compare-gvo3eind` (birleşik debug) ve `compare-wmabbl5t` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli üç eksenli map ve üç box girdisiyle sınırlıdır. Özel merkez overload ayrı [all-axis merkez parity belgesinde](shape-rotate-all-axis-center-parity.md) doğrulanır. Merkez parametreli diğer overload'lar, `rotateHorizontal`, `rotateAll` yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
