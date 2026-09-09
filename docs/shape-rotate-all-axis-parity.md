# P01.d — Shapes.rotateAllAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis` yolunu X, Y ve Z eksen varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 559 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **559/559** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, üç dönüşün sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1013/1013** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `2809154f292cf47ecd8113f8c4766b7a0af297d2ab88e185e421bdfac514e9c9`.

Birleşik referans kimliği: `8bd229a580fc01d7c478c33923afb04c0fee7803869b7423f2d3ad4587821b9f`.

Yerel raporlar: `compare-tl6kn3g9` (math debug), `compare-uges__2a` (math release), `compare-n_x78ct2` (birleşik debug) ve `compare-pzl7bv8b` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli üç eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload, `rotateHorizontal`, `rotateAll` yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
