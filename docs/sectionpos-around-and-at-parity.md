# P01.c — SectionPos aroundAndAtBlockPos doğrulaması

`src/binary.rs`, SectionPos `aroundAndAtBlockPos` üç overload’unu Java ile aynı section node listesine ve sectionX/sectionY/sectionZ callback sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos aroundAndAtBlockPos için 6 özgün senaryo hazırlandı.
- coordinates, BlockPos ve packed overload’ları; interior, sınır, köşe ve i32 taşması doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1249/1249** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`. Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Adapter hashleri: debug `9437f241df809f7c41acbf27ebf40a730cba810baaebd620ac0006230687c3d9`, release `2749c23cb3aa8019363af01df229657d8d954faa4a8063bc6cfcc0ce5f11820d`.

Yerel raporlar: `compare-dj1scyn3` (math debug), `compare-1g72i2us` (math release), `compare-f0bjlm2c` (birleşik debug), `compare-eaqfihsj` (birleşik release), `verify-_qy4rx_r` (math) ve `verify-p2v9lu2o` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.aroundAndAtBlockPos(BlockPos)`, packed long ve coordinate overload’larının küçük finite node listesi çıktısıyla sınırlıdır. `blocksInside`, range helpers, stream codec ve callback/laziness özellikleri ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
