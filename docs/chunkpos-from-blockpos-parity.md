# P01.c — ChunkPos BlockPos'tan containing/pack doğrulaması

`src/binary.rs`, Java `ChunkPos.containing(BlockPos)` ve `ChunkPos.pack(BlockPos)` davranışlarını x/z arithmetic shift'i, y'nin dışarıda bırakılması ve packed 32 bit alan yerleşimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed BlockPos girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- BlockPos'tan containing/pack için 6 özgün senaryo hazırlandı.
- Pozitif ve negatif 16'lık sınırlar, mixed x/z, signed i32 uçları, y'nin etkisizliği ve packed unsigned hex metni Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1378/1378** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1880 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`. Birleşik referans kimliği: `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81`.

Yerel raporlar: `compare-_ohp5wb8` (math debug), `compare-bezugwpw` (math release), `compare-i4o7vofo` (birleşik debug), `compare-9vhvfokc` (birleşik release) ve `verify-kfxghdw3` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, BlockPos'tan ChunkPos containing ve packed long üretimini kapsar. BlockPos packing'i, SectionPos node packing'i, diğer ChunkPos codec/yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
