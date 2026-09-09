# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1013/1013** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1515/1515** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `5ce242e52e41e1869ca609121bb59c3acf18c2e75ba193e14bf67ead2c12ed7b`. Birleşik referans kimliği: `b6d98de9faec99fa12618268a20692921e10931fc251b87964810964c0de2933`.

Adapter hashleri: debug `523bcc34745739a7c25bb74f50b215c5eced318db4a8cc8ecabceea1c4a4982f`, release `8252fa4a063bd5eb2e8f4d634b0ecc06e072000500fca7dd2bda1f3b36670acc`.

Yerel raporlar: `compare-dsrceuzh` ve `compare-2cvy6vt1` (math debug/release), `compare-5omvt90q` ve `compare-orlrlnqo` (birleşik debug/release), `verify-4bc3l5fr` (math) ve `verify-wa8bpydo` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong(BlockPos)`, `offset`, `getZeroNode` dışındaki node akışları ve `blocksInside`/`cube` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
