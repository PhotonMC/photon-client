# P01.c — SectionPos spatial doğrulaması

`src/binary.rs`, SectionPos `origin()`, `center()` ve `chunk()` sonuçlarını Java ile aynı BlockPos ve ChunkPos değerlerine dönüştürür.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos spatial yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1007/1007** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`. Birleşik referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Adapter hashleri: debug `c6d08d9b531efca811e427368bf0304d2f3f9cadf66f40df9eeb629de4210b9a`, release `f91d8327e0ec7dc56843ae1dfd8cf5986ff2c894c5b1921e532c6bb0190cce72`.

Yerel raporlar: `compare-upfzoh9c` ve `compare-czpd1pgo` (math debug/release), `compare-_3h0xf61` ve `compare-a2h2wcm2` (birleşik debug/release), `verify-lyvtnknl` (math) ve `verify-19j4rwux` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos origin/center/chunk dönüşümleriyle sınırlıdır. SectionPos blockToSection, zero node, sectionToChunk packed yolları ve blocksInside/cube akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
