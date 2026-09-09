# P01.c — SectionPos relative-to-block doğrulaması

`src/binary.rs`, SectionPos içindeki short relative ofsetini Java `sectionRelativeX/Y/Z` ve `relativeToBlockX/Y/Z` yollarıyla aynı sonuçlara çevirir; `relativeToBlockPos` da aynı üç bileşeni BlockPos'a taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos relative-to-block yardımcıları için 6 özgün senaryo hazırlandı.
- Sıfır/köşe ofsetleri, signed short bit desenleri, karışık alanlar ve i32 koordinat taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1249/1249** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1751/1751** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `29be5693737a944f40d9534d4a126556f5aeaac24dd9d2f5fccd0c6b1db3a3aa`. Birleşik referans kimliği: `a92e7ef25d7ffcf101102bab4b05061d96a3b0511495a77464f282a0775abfd0`.

Adapter hashleri: debug `2e1ed7c23719ce14a76b19ea36b422abdc2629c325d726e2e6dc4a0b2542924d`, release `8bbbe1c0f9dd455565fc7c1cc8884bf41ae0c56ee1b3a6a8e91a1facb40a230b`.

Yerel raporlar: `compare-dj1scyn3` ve `compare-1g72i2us` (math debug/release), `compare-f0bjlm2c` ve `compare-eaqfihsj` (birleşik debug/release), `verify-_qy4rx_r` (math) ve `verify-p2v9lu2o` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos short relative ofsetinin section içi blok koordinatlarına çevrilmesiyle sınırlıdır. SectionPos origin/center/chunk dönüşümleri, blockToSection ve packed üretim yolları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
