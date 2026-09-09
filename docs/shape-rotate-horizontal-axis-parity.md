# P01.d — Shapes.rotateHorizontalAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateHorizontalAxis` yolunu X ve Z eksen varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 543 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **543/543** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Z map anahtarlarını, dönüş sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **997/997** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `63801053ac2dac92dacdf64714ccc1991d239550a9fdd9f44c2463b2a3897842`.

Birleşik referans kimliği: `d12db998718af31adab6351bbe129bcfd8d13fb33be8aa4270b7bc0546294c2d`.

Yerel raporlar: `compare-uwkm_8cp` (math debug), `compare-73jkknpc` (math release), `compare-m_cdl8w8` (birleşik debug) ve `compare-xp7qipgf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli iki eksenli map ve üç box girdisiyle sınırlıdır. `rotateAllAxis`, yön map'leri, merkez parametreli overload'lar ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
