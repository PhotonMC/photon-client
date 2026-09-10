# P01.c — SectionPos aroundAndAtBlockPos doğrulaması

`src/binary.rs`, SectionPos `aroundAndAtBlockPos` üç overload’unu Java ile aynı section node listesine ve sectionX/sectionY/sectionZ callback sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos aroundAndAtBlockPos için 6 özgün senaryo hazırlandı.
- coordinates, BlockPos ve packed overload’ları; interior, sınır, köşe ve i32 taşması doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1783/1783** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`. Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Adapter hashleri: debug `843cc0c04c00803acc1e8bf774f290df5e273c7777789a8df5b971cb6a7f1ec0`, release `200c25d47d60d5c096012435e86b4b9f28f06965fa1ac6a453b948b2e6e33f4e`.

Yerel raporlar: `compare-1g27oorn` (math debug), `compare-bkcvw7oi` (math release), `compare-qmmqnzwk` (birleşik debug), `compare-ogma6668` (birleşik release), `verify-_e108v7f` (math) ve `verify-1hv399w4` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.aroundAndAtBlockPos(BlockPos)`, packed long ve coordinate overload’larının küçük finite node listesi çıktısıyla sınırlıdır. `blocksInside`, range helpers, stream codec ve callback/laziness özellikleri ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
