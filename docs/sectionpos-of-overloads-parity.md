# P01.c — SectionPos of(...) overloadları doğrulaması

`src/binary.rs`, SectionPos'u BlockPos, ChunkPos ve double Position girdilerinden üretme yollarını ve packed SectionPos açılımını Java ile aynı section koordinatlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos `of(...)` veri overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos signed i32 sınırları, ChunkPos + sectionY aktarımı, packed alan sign extension'ı, negatif/pozitif double floor ve özel double değerleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1743/1743** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`. Birleşik referans kimliği: `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968`.

Adapter hashleri: debug `65b6a51802401fd8e16a2ce346287fded6541bdbb762922fa5ed76876f2007f4`, release `99b722f883332e4576e669f7db57f98dac937e328b3a669185056e37f8dec786`.

Yerel raporlar: `compare-q3ryrs_k` ve `compare-xtfhknri` (math debug/release), `compare-ag5oxl30` ve `compare-fkrk2srf` (birleşik debug/release), `verify-hvezlnv1` (math) ve `verify-lstfjdye` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarının son SectionPos snapshot'ıyla sınırlıdır. `of(EntityAccess)`, `bottomOf(ChunkAccess)`, gerçek entity/chunk üretim zincirleri, stream codec ve `of(int, int, int)` için ayrı çağrı gözlemi bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
