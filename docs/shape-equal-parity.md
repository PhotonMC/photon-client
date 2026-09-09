# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md), merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 571 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **571/571** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1025/1025** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `bf8eac3fe88ccc7eac0052e8b44cb5dab3dfb456982cd6a700ad1f08fadc02a2`.

Birleşik referans kimliği: `3133f0b82eddafd4383cd19cad3d4e53db2b9051614cfc78968ee4d6d98810b8`.

Yerel raporlar: `compare-wbo29fbq` (math debug), `compare-7m5csdkl` (math release), `compare-tyoie69t` (birleşik debug) ve `compare-_lv2ppe_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
