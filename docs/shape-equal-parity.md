# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md), merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 577 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **577/577** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1031/1031** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5df31dedf41786bcd5bb38fc117404df6a83da03bce464c1a637063e5b3b5cbb`.

Birleşik referans kimliği: `cc0f402235532cc834549c53fd069f064a8e771ef6ba6e2943e472b1d9b6b06d`.

Yerel raporlar: `compare-jrhf0iba` (math debug), `compare-8berq09t` (math release), `compare-rpvlapca` (birleşik debug) ve `compare-dwzkalq0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
