# P01.c — ChunkPos BlockPos'tan containing/pack doğrulaması

`src/binary.rs`, Java `ChunkPos.containing(BlockPos)` ve `ChunkPos.pack(BlockPos)` davranışlarını x/z arithmetic shift'i, y'nin dışarıda bırakılması ve packed 32 bit alan yerleşimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed BlockPos girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- BlockPos'tan containing/pack için 6 özgün senaryo hazırlandı.
- Pozitif ve negatif 16'lık sınırlar, mixed x/z, signed i32 uçları, y'nin etkisizliği ve packed unsigned hex metni Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2578 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Yerel raporlar: `compare-k2v65l83` (math debug), `compare-9uf1gox2` (math release), `compare-ptlm9ptr` (birleşik debug), `compare-86z18x_k` (birleşik release) ve `verify-31k8bjcu` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, BlockPos'tan ChunkPos containing ve packed long üretimini kapsar. BlockPos packing'i, SectionPos node packing'i, diğer ChunkPos codec/yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
