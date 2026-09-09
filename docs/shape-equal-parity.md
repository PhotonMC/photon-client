# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar. `Shapes.rotate` sonucu ayrı [rotation kanıtında](shape-rotate-parity.md), merkez overload sonucu [merkez rotation kanıtında](shape-rotate-center-parity.md), `Shapes.rotateHorizontalAxis` sonucu [yatay eksen kanıtında](shape-rotate-horizontal-axis-parity.md) ve `Shapes.rotateAllAxis` sonucu [all-axis kanıtında](shape-rotate-all-axis-parity.md), `Shapes.rotateHorizontal` sonucu [horizontal kanıtında](shape-rotate-horizontal-parity.md), `Shapes.rotateAll` sonucu [all-direction kanıtında](shape-rotate-all-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 580 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **580/580** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1034/1034** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8db71332cfbcc696bf89823167fb052b36cf4e81b874256da28436326d9cdadc`.

Birleşik referans kimliği: `d912c38089b8e4b8d1bbab5d2d9ad532a60108e293727c14cc25d6f3ad0e7333`.

Yerel raporlar: `compare-cgdwsv4p` (math debug), `compare-ydb0q8z5` (math release), `compare-guo5917x` (birleşik debug) ve `compare-jlxs6tma` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
