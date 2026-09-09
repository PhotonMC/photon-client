# P01.c — SectionPos relative-to-block doğrulaması

`src/binary.rs`, SectionPos içindeki short relative ofsetini Java `sectionRelativeX/Y/Z` ve `relativeToBlockX/Y/Z` yollarıyla aynı sonuçlara çevirir; `relativeToBlockPos` da aynı üç bileşeni BlockPos'a taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos relative-to-block yardımcıları için 6 özgün senaryo hazırlandı.
- Sıfır/köşe ofsetleri, signed short bit desenleri, karışık alanlar ve i32 koordinat taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1319/1319** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1821/1821** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `85b9ce09336ec312c84a062e772cdb41b6a2e079b6fcb1995200470edf61dd6c`. Birleşik referans kimliği: `01a770901948bb0f5650c8b4e80bf54b513d904c9746232c38b93c03a7546957`.

Adapter hashleri: debug `2e1ed7c23719ce14a76b19ea36b422abdc2629c325d726e2e6dc4a0b2542924d`, release `8bbbe1c0f9dd455565fc7c1cc8884bf41ae0c56ee1b3a6a8e91a1facb40a230b`.

Yerel raporlar: `compare-stkj42d2` ve `compare-e9upg098` (math debug/release), `compare-olmqgxpc` ve `compare-jxg0__tj` (birleşik debug/release), `verify-6wifh_w5` (math) ve `verify-m0natom3` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos short relative ofsetinin section içi blok koordinatlarına çevrilmesiyle sınırlıdır. SectionPos origin/center/chunk dönüşümleri, blockToSection ve packed üretim yolları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
