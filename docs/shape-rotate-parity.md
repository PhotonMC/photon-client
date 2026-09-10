# P01.d — Shapes.rotate doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.rotate` yolunun blok merkezli signed eksen permütasyonunu, koordinat düzlemlerinin merkez etrafında yansıtılmasını ve discrete dolu hücre indislerinin taşınmasını uygular. `examples/behavior_adapter.rs` 12 yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotateHorizontalAxis` sonucu ayrı [axis parity belgesinde](shape-rotate-horizontal-axis-parity.md), `Shapes.rotateAllAxis` sonucu [all-axis parity belgesinde](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal parity belgesinde](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction parity belgesinde](shape-rotate-all-parity.md), merkez overload sonucu [merkez rotation parity belgesinde](shape-rotate-center-parity.md), Shapes.rotateAttachFace sonucu [attach-face parity belgesinde](shape-rotate-attach-face-parity.md), Shapes.rotateHorizontal initial sonucu [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md), Shapes.rotateAll initial sonucu [all-direction initial parity belgesinde](shape-rotate-all-initial-parity.md), Shapes.rotateAttachFace initial sonucu [attach-face initial parity belgesinde](shape-rotate-attach-face-initial-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1743 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1743/1743** math senaryosunda eşleşti.
- On tek-box dönüşü; identity, negatif/pozitif 90 derece dönüşler ve üç 180 derece yüz dönüşünü karşılaştırdı.
- Grid boyutu ve OR ile birleştirilmiş hücre senaryoları, yalnızca koordinat sınırlarını değil discrete hücre dönüşümünü ve `toAabbs` sırasını da karşılaştırdı.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Yerel raporlar: `compare-q3ryrs_k` (math debug), `compare-xtfhknri` (math release), `compare-ag5oxl30` (birleşik debug) ve `compare-fkrk2srf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt identity ve `OctahedralGroup` içindeki dokuz non-identity dönüş adı ile box/OR shape girdileriyle sınırlıdır. Merkez parametreli overload ayrı [merkez rotation parity belgesinde](shape-rotate-center-parity.md) doğrulanır; diğer shape kaynakları ve world face/collider toplama bu dilimin kapsamı değildir.
