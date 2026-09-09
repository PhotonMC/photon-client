# P01.c — SectionPos blocksInside doğrulaması

`src/binary.rs`, SectionPos `blocksInside()` akışını Java ile aynı 16×16×16 inclusive BlockPos küpüne ve x/y/z traversal sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blocksInside için 3 özgün senaryo hazırlandı.
- Her senaryo 4096 BlockPos çıktısında origin, negatif section ve section→block sınırlarını doğrudan Java/Rust API'leriyle karşılaştırdı.
- Rust debug ve release adaptörleri güncel **1249/1249** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`. Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Adapter hashleri: debug `9437f241df809f7c41acbf27ebf40a730cba810baaebd620ac0006230687c3d9`, release `2749c23cb3aa8019363af01df229657d8d954faa4a8063bc6cfcc0ce5f11820d`.

Yerel raporlar: `compare-dj1scyn3` ve `compare-1g72i2us` (math debug/release), `compare-f0bjlm2c` ve `compare-eaqfihsj` (birleşik debug/release), `verify-_qy4rx_r` (math) ve `verify-p2v9lu2o` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.blocksInside()` ve kullandığı `BlockPos.betweenClosedStream` yolunun sabit 4096 öğelik çıktısıyla sınırlıdır. Genel BlockPos range overload'ları, stream laziness/spliterator özellikleri ve `aroundAndAtBlockPos` akışı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
