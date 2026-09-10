# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-horizontal-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` varsayılan üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Yerel raporlar: `compare-hiau72j0` (math debug), `compare-bttpme0o` (math release), `compare-k17hah5m` (birleşik debug) ve `compare-hiiz6u35` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload ayrı [yatay eksen merkez parity belgesinde](shape-rotate-horizontal-axis-center-parity.md) doğrulanır; `rotateAllAxis` sonucu ayrı [all-axis parity belgesinde](shape-rotate-all-axis-parity.md) doğrulanır; yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
