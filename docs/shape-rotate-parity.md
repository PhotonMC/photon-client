# P01.d — Shapes.rotate doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 540 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **540/540** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **994/994** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `075dce9be8f50635713e1d727226fab4db4a932649719fbc7f997af615335d07`.

Birleşik referans kimliği: `1ad3b7dff5eb6494fe2bfe71adb63a9beb68d01b337e297f3838e37518f3d62f`.

Yerel raporlar: `compare-83oig85q` (math debug), `compare-wde1o1u_` (math release), `compare-yb206b7s` (birleşik debug) ve `compare-og49xhho` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload, diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
