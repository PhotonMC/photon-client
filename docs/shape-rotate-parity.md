# P01.d — Shapes.rotate doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotateHorizontalAxis` sonucu ayrı [axis parity belgesinde](shape-rotate-horizontal-axis-parity.md), `Shapes.rotateAllAxis` sonucu [all-axis parity belgesinde](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal parity belgesinde](shape-rotate-horizontal-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 549 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **549/549** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **1003/1003** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `f205ceba714d4702c836571f170f9dfb4fd0000df6c1d6a58b646deb7d6c4426`.

Birleşik referans kimliği: `087a802cf2b50f4fd723cf119b31303a75b2a07d7e98062d8b40b0fa9394d3af`.

Yerel raporlar: `compare-2c90a5am` (math debug), `compare-4j73hu_g` (math release), `compare-xm1dqtf3` (birleşik debug) ve `compare-1fuiwzut` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload, diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
