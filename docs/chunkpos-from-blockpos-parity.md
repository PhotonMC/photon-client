# P01.c — ChunkPos BlockPos'tan containing/pack doğrulaması

`src/binary.rs`, Java `ChunkPos.containing(BlockPos)` ve `ChunkPos.pack(BlockPos)` davranışlarını x/z arithmetic shift'i, y'nin dışarıda bırakılması ve packed 32 bit alan yerleşimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed BlockPos girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- BlockPos'tan containing/pack için 6 özgün senaryo hazırlandı.
- Pozitif ve negatif 16'lık sınırlar, mixed x/z, signed i32 uçları, y'nin etkisizliği ve packed unsigned hex metni Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **1221/1221** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1723 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`. Birleşik referans kimliği: `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf`.

Yerel raporlar: `compare-6dvmyv_h` (math debug), `compare-ak0n9_u3` (math release), `compare-_8esk0jy` (birleşik debug), `compare-g98k9sn0` (birleşik release) ve `verify-72fv37y0` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, BlockPos'tan ChunkPos containing ve packed long üretimini kapsar. BlockPos packing'i, SectionPos node packing'i, diğer ChunkPos codec/yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
