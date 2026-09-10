# P01.c — SectionPos spatial doğrulaması

`src/binary.rs`, SectionPos `origin()`, `center()` ve `chunk()` sonuçlarını Java ile aynı BlockPos ve ChunkPos değerlerine dönüştürür.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos spatial yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1608/1608** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`. Birleşik referans kimliği: `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3`.

Adapter hashleri: debug `093dadfb72b58cdb548dc86ac07d1392b14d7d9d30e2dfc064973a213e00e8da`, release `35db30ea7c4bfc3b9d9c2e45950e6a7eefa07975b15170c80a9ade30c5eb5bbe`.

Yerel raporlar: `compare-pmcommj0` ve `compare-4g5l34ea` (math debug/release), `compare-mki3lw11` ve `compare-r66knife` (birleşik debug/release), `verify-_islsta6` (math) ve `verify-35w0iz71` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos origin/center/chunk dönüşümleriyle sınırlıdır. SectionPos blockToSection, zero node, sectionToChunk packed yolları ve blocksInside/range/aroundAndAtBlockPos akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
