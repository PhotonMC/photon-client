# P01.d — Shapes.rotateAllAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis` yolunu X, Y ve Z eksen varyantlarıyla uygular; özel merkez overload sonucu ayrı [kanıtta](shape-rotate-all-axis-center-parity.md) doğrulanır. `examples/behavior_adapter.rs` üç JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 627 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **627/627** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, üç dönüşün sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1081/1081** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Birleşik referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Yerel raporlar: `compare-tg0rl658` (math debug), `compare-713odvz2` (math release), `compare-pnvz2myv` (birleşik debug) ve `compare-_bmqdjjb` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli üç eksenli map ve üç box girdisiyle sınırlıdır. Özel merkez overload ayrı [all-axis merkez parity belgesinde](shape-rotate-all-axis-center-parity.md) doğrulanır. Merkez parametreli diğer overload'lar, `rotateHorizontal`, `rotateAll` yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
