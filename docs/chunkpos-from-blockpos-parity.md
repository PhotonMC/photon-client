# P01.c — ChunkPos BlockPos'tan containing/pack doğrulaması

`src/binary.rs`, Java `ChunkPos.containing(BlockPos)` ve `ChunkPos.pack(BlockPos)` davranışlarını x/z arithmetic shift'i, y'nin dışarıda bırakılması ve packed 32 bit alan yerleşimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed BlockPos girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- BlockPos'tan containing/pack için 6 özgün senaryo hazırlandı.
- Pozitif ve negatif 16'lık sınırlar, mixed x/z, signed i32 uçları, y'nin etkisizliği ve packed unsigned hex metni Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1578/1578** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2080/2080** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2080 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `845bee8faa9bd67215b4c4a330fee9163ef37c01793940d730ec7655ee66c654`. Birleşik referans kimliği: `e1fbdb47e2227938261f4e5bd3238926120301582d50b1af7c84fefac9844a94`.

Yerel raporlar: `compare-l4cf39d7` (math debug), `compare-jvzkn11s` (math release), `compare-oc9sa8gq` (birleşik debug), `compare-m9ue0q18` (birleşik release) ve `verify-np1asir2` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, BlockPos'tan ChunkPos containing ve packed long üretimini kapsar. BlockPos packing'i, SectionPos node packing'i, diğer ChunkPos codec/yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
