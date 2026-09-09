# P01.c — SectionPos of(...) overloadları doğrulaması

`src/binary.rs`, SectionPos'u BlockPos, ChunkPos ve double Position girdilerinden üretme yollarını ve packed SectionPos açılımını Java ile aynı section koordinatlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos `of(...)` veri overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos signed i32 sınırları, ChunkPos + sectionY aktarımı, packed alan sign extension'ı, negatif/pozitif double floor ve özel double değerleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1193/1193** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `3a6a2f5a3d8ff6b65e565413ca95aeeb802f3ca965620ed874e898bcc52c5d61`. Birleşik referans kimliği: `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c`.

Adapter hashleri: debug `632535c3aabe8ce4f0613f6b855c54e499456585aa71beaa93412b83b0c57a54`, release `7c059afa0db707e2c536034aab21d80bc8d27f9b04cbe81728a84261b43691a7`.

Yerel raporlar: `compare-dgq7mp_7` ve `compare-2hmy86uj` (math debug/release), `compare-06i9lguy` ve `compare-r6guli2r` (birleşik debug/release), `verify-9n41v5aa` (math) ve `verify-k_vt31c0` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarının son SectionPos snapshot'ıyla sınırlıdır. `of(EntityAccess)`, `bottomOf(ChunkAccess)`, gerçek entity/chunk üretim zincirleri, stream codec ve `of(int, int, int)` için ayrı çağrı gözlemi bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
