# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md), merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb`.

Birleşik referans kimliği: `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb`.

Yerel raporlar: `compare-d1bitq7h` (math debug), `compare-ldlepu8u` (math release), `compare-kuzkqdbl` (birleşik debug) ve `compare-n3x2kj9p` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
