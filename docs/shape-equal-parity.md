# P01.d — Shapes.equal doğrulaması

`src/shape.rs`, Java 26.2 `Shapes.equal` davranışını `NOT_SAME` birleşiminin boşluk sorgusunu tersleyerek uygular. `examples/behavior_adapter.rs` altı yeni JSONL senaryosunu bu metoda bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 528 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **528/528** math senaryosunda eşleşti.
- Yeni `Shapes.equal` senaryoları aynı box, aynı taşınmış box, farklı boyut, sınırdan temas, kısmi örtüşme ve farklı konumlardaki iki empty shape sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **982/982** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `75eb7e188d3237557080017eae7056f68a6cb46cda4d41b3a7670507842ff5ae`.

Birleşik referans kimliği: `132874b38edd2ac3df4e52525cdc4defbd5f36ee22441bb8daba624d5c98076c`.

Yerel raporlar: `compare-v8iab_92` (math debug), `compare-ecl7shzk` (math release), `compare-hz_hpajw` (birleşik debug) ve `compare-7a4v0a9c` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt box tabanlı altı senaryoyla sınırlıdır. Shape rotation, daha geniş world face/collider toplama ve nesne/hash API'leri bu dilimin kapsamı değildir.
