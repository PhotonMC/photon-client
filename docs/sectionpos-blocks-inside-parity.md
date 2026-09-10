# P01.c — SectionPos blocksInside doğrulaması

`src/binary.rs`, SectionPos `blocksInside()` akışını Java ile aynı 16×16×16 inclusive BlockPos küpüne ve x/y/z traversal sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blocksInside için 3 özgün senaryo hazırlandı.
- Her senaryo 4096 BlockPos çıktısında origin, negatif section ve section→block sınırlarını doğrudan Java/Rust API'leriyle karşılaştırdı.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Adapter hashleri: debug `1a871a434c144a587a2723327bd46aaf9a6d1b4612396a66fffe175369c31d12`, release `8c7871587c66a581ea4b33d57a1ef23281843488a7661885a1a704dee0ad3e74`.

Yerel raporlar: `compare-qtqxwt9b` ve `compare-ebava42g` (math debug/release), `compare-x0hkkvkv` ve `compare-x5wgwswa` (birleşik debug/release), `verify-m2eek8to` (math) ve `verify-rc21vyy9` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.blocksInside()` ve kullandığı `BlockPos.betweenClosedStream` yolunun sabit 4096 öğelik çıktısıyla sınırlıdır. Genel BlockPos range overload'ları, stream laziness/spliterator özellikleri ve `aroundAndAtBlockPos` akışı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
