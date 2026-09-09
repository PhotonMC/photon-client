# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-horizontal-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` varsayılan üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 765 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **765/765** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1219/1219** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a`.

Birleşik referans kimliği: `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a`.

Yerel raporlar: `compare-ya8jvd1o` (math debug), `compare-rh0c0wxr` (math release), `compare-u_e3yyct` (birleşik debug) ve `compare-326ehkgq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload ayrı [yatay eksen merkez parity belgesinde](shape-rotate-horizontal-axis-center-parity.md) doğrulanır; `rotateAllAxis` sonucu ayrı [all-axis parity belgesinde](shape-rotate-all-axis-parity.md) doğrulanır; yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
