# P01.d — Shapes.rotateAllAxis doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotateAllAxis` yolunu X, Y ve Z eksen varyantlarıyla uygular; `examples/behavior_adapter.rs` üç yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 546 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **546/546** math senaryosunda eşleşti.
- Asimetrik, yarım grid boyutlu ve empty shape girdileri X/Y/Z map anahtarlarını, üç dönüşün sınırlarını, hücre snapshot'larını ve EnumMap sırasını karşılaştırdı.
- Birleşik altı aile corpus'u **1000/1000** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `b8df21c506d300a30947ce77e21841210f33991f41bd8bbdf4dd4a1ab775a343`.

Birleşik referans kimliği: `b624d0ee4bd5155be0aa8e190a91cc25d84867d5c6f605db7c91ad8c599196f2`.

Yerel raporlar: `compare-pxfrslbe` (math debug), `compare-le8qojd3` (math release), `compare-ag226u45` (birleşik debug) ve `compare-8116vkuu` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt varsayılan merkezli üç eksenli map ve üç box girdisiyle sınırlıdır. Merkez parametreli overload, `rotateHorizontal`, `rotateAll` yön map'leri ve diğer `OctahedralGroup` değerleri ayrı dilimlerdir.
