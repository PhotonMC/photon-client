# P01.c — SectionPos relative-to-block doğrulaması

`src/binary.rs`, SectionPos içindeki short relative ofsetini Java `sectionRelativeX/Y/Z` ve `relativeToBlockX/Y/Z` yollarıyla aynı sonuçlara çevirir; `relativeToBlockPos` da aynı üç bileşeni BlockPos'a taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos relative-to-block yardımcıları için 6 özgün senaryo hazırlandı.
- Sıfır/köşe ofsetleri, signed short bit desenleri, karışık alanlar ve i32 koordinat taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1335/1335** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`. Birleşik referans kimliği: `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9`.

Adapter hashleri: debug `2e1ed7c23719ce14a76b19ea36b422abdc2629c325d726e2e6dc4a0b2542924d`, release `8bbbe1c0f9dd455565fc7c1cc8884bf41ae0c56ee1b3a6a8e91a1facb40a230b`.

Yerel raporlar: `compare-0dy1p_jx` ve `compare-fdzx3gij` (math debug/release), `compare-gridp_gxu` ve `compare-u8zwfn4r` (birleşik debug/release), `verify-cwah2qjd` (math) ve `verify-yhwxvq8l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos short relative ofsetinin section içi blok koordinatlarına çevrilmesiyle sınırlıdır. SectionPos origin/center/chunk dönüşümleri, blockToSection ve packed üretim yolları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
