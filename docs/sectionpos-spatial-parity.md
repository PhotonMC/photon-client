# P01.c — SectionPos spatial doğrulaması

`src/binary.rs`, SectionPos `origin()`, `center()` ve `chunk()` sonuçlarını Java ile aynı BlockPos ve ChunkPos değerlerine dönüştürür.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos spatial yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1534/1534** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`. Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Adapter hashleri: debug `c0ccef8f56e8193f0ee8d76b66cee34ceb84310952f2f6bced3e9565c3fc3a36`, release `371d975bc049366bd9dc9f0d528455416d0fc508d885380fe5fa239d21082f7d`.

Yerel raporlar: `compare-t6yvipac` ve `compare-6_snmnk3` (math debug/release), `compare-tgx0qoj4` ve `compare-t1bsi_04` (birleşik debug/release), `verify-qesbmcro` (math) ve `verify-zwp9iban` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos origin/center/chunk dönüşümleriyle sınırlıdır. SectionPos blockToSection, zero node, sectionToChunk packed yolları ve blocksInside/range/aroundAndAtBlockPos akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
