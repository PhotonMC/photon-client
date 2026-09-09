# P01.d — Shapes.rotate doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotateHorizontalAxis` sonucu ayrı [axis parity belgesinde](shape-rotate-horizontal-axis-parity.md), `Shapes.rotateAllAxis` sonucu [all-axis parity belgesinde](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal parity belgesinde](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction parity belgesinde](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 552 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **552/552** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **1006/1006** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5161df0d1744dc53eed78d5785d13024594d5b8306cdcf50f765118ae2042450`.

Birleşik referans kimliği: `bbcdd8d522a3e0799b6945441c0c42cd1e0c98bbb8c9dceda370d0203bed11e9`.

Yerel raporlar: `compare-941j34iv` (math debug), `compare-73_p7rtu` (math release), `compare-88gexq6y` (birleşik debug) ve `compare-dn88muge` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload, diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
