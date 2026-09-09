# P01.d — Shapes.rotate doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotateHorizontalAxis` sonucu ayrı [axis parity belgesinde](shape-rotate-horizontal-axis-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 543 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **543/543** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **997/997** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `63801053ac2dac92dacdf64714ccc1991d239550a9fdd9f44c2463b2a3897842`.

Birleşik referans kimliği: `d12db998718af31adab6351bbe129bcfd8d13fb33be8aa4270b7bc0546294c2d`.

Yerel raporlar: `compare-uwkm_8cp` (math debug), `compare-73jkknpc` (math release), `compare-m_cdl8w8` (birleşik debug) ve `compare-xp7qipgf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload, diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
