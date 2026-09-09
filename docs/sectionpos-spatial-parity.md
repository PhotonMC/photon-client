# P01.c — SectionPos spatial doğrulaması

`src/binary.rs`, SectionPos `origin()`, `center()` ve `chunk()` sonuçlarını Java ile aynı BlockPos ve ChunkPos değerlerine dönüştürür.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos spatial yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1013/1013** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1515/1515** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `5ce242e52e41e1869ca609121bb59c3acf18c2e75ba193e14bf67ead2c12ed7b`. Birleşik referans kimliği: `b6d98de9faec99fa12618268a20692921e10931fc251b87964810964c0de2933`.

Adapter hashleri: debug `523bcc34745739a7c25bb74f50b215c5eced318db4a8cc8ecabceea1c4a4982f`, release `8252fa4a063bd5eb2e8f4d634b0ecc06e072000500fca7dd2bda1f3b36670acc`.

Yerel raporlar: `compare-dsrceuzh` ve `compare-2cvy6vt1` (math debug/release), `compare-5omvt90q` ve `compare-orlrlnqo` (birleşik debug/release), `verify-4bc3l5fr` (math) ve `verify-wa8bpydo` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos origin/center/chunk dönüşümleriyle sınırlıdır. SectionPos blockToSection, zero node, sectionToChunk packed yolları ve blocksInside/cube akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
