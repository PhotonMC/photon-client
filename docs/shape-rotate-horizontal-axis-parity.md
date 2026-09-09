# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-horizontal-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` varsayılan üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1007 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1007/1007** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Birleşik referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug) ve `compare-a2h2wcm2` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload ayrı [yatay eksen merkez parity belgesinde](shape-rotate-horizontal-axis-center-parity.md) doğrulanır; `rotateAllAxis` sonucu ayrı [all-axis parity belgesinde](shape-rotate-all-axis-parity.md) doğrulanır; yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
