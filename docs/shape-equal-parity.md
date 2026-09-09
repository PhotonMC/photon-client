# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md), merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 601 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **601/601** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1055/1055** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `ea03d29d513eca99de7f40ea68c6623f22649f5ce9a4d6dc4ef0a5ac02cb63f3`.

Birleşik referans kimliği: `716f0b454b3848994e9afd84c60130e20d821311c22b7c1e80590008852fda52`.

Yerel raporlar: `compare-8dsunddp` (math debug), `compare-zvv3a_3_` (math release), `compare-9k3hskia` (birleşik debug) ve `compare-yw9njwwe` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
