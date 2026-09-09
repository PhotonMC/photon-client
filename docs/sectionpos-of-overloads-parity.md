# P01.c — SectionPos of(...) overloadları doğrulaması

`src/binary.rs`, SectionPos'u BlockPos, ChunkPos ve double Position girdilerinden üretme yollarını ve packed SectionPos açılımını Java ile aynı section koordinatlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos `of(...)` veri overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos signed i32 sınırları, ChunkPos + sectionY aktarımı, packed alan sign extension'ı, negatif/pozitif double floor ve özel double değerleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1067/1067** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`. Birleşik referans kimliği: `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1`.

Adapter hashleri: debug `2e569fd5b282fc3c6202cf4a505416139c9e9cf90f6ebc711c0851970da8bc15`, release `72e7c71370f232b44263e273b357aab840c978acec80458d8e02a7b31aef41a1`.

Yerel raporlar: `compare-d3vh9fy0` ve `compare-cdl_uz9p` (math debug/release), `compare-b4u1_5xx` ve `compare-lz2txc82` (birleşik debug/release), `verify-cmxttwgw` (math) ve `verify-i4bptixz` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarının son SectionPos snapshot'ıyla sınırlıdır. `of(EntityAccess)`, `bottomOf(ChunkAccess)`, gerçek entity/chunk üretim zincirleri, stream codec ve `of(int, int, int)` için ayrı çağrı gözlemi bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
