# P01.c — SectionPos of(...) overloadları doğrulaması

`src/binary.rs`, SectionPos'u BlockPos, ChunkPos ve double Position girdilerinden üretme yollarını ve packed SectionPos açılımını Java ile aynı section koordinatlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos `of(...)` veri overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos signed i32 sınırları, ChunkPos + sectionY aktarımı, packed alan sign extension'ı, negatif/pozitif double floor ve özel double değerleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1378/1378** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`. Birleşik referans kimliği: `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81`.

Adapter hashleri: debug `9a363e9bb616ddf43015a46c9dac5fe7c3f1f0a485172346278b402b743e4e87`, release `afe7371264512dc68a7bfa00996d3d561cca30b346b8edb775b90e213ad3d397`.

Yerel raporlar: `compare-_ohp5wb8` ve `compare-bezugwpw` (math debug/release), `compare-i4o7vofo` ve `compare-9vhvfokc` (birleşik debug/release), `verify-i3bunamx` (math) ve `verify-kfxghdw3` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarının son SectionPos snapshot'ıyla sınırlıdır. `of(EntityAccess)`, `bottomOf(ChunkAccess)`, gerçek entity/chunk üretim zincirleri, stream codec ve `of(int, int, int)` için ayrı çağrı gözlemi bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
