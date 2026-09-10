# P01.c — SectionPos relative-to-block doğrulaması

`src/binary.rs`, SectionPos içindeki short relative ofsetini Java `sectionRelativeX/Y/Z` ve `relativeToBlockX/Y/Z` yollarıyla aynı sonuçlara çevirir; `relativeToBlockPos` da aynı üç bileşeni BlockPos'a taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos relative-to-block yardımcıları için 6 özgün senaryo hazırlandı.
- Sıfır/köşe ofsetleri, signed short bit desenleri, karışık alanlar ve i32 koordinat taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1763/1763** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `7e0ba63794c7ec98b0e5be4494e1fd999edd354071389371ea9497d0096242aa`. Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Adapter hashleri: debug `2e1ed7c23719ce14a76b19ea36b422abdc2629c325d726e2e6dc4a0b2542924d`, release `8bbbe1c0f9dd455565fc7c1cc8884bf41ae0c56ee1b3a6a8e91a1facb40a230b`.

Yerel raporlar: `compare-7mgwi148` ve `compare-rq3_yvog` (math debug/release), `compare-efi7e5qh` ve `compare-qw_kmael` (birleşik debug/release), `verify-tljqi7wo` (math) ve `verify-oxhrmmem` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos short relative ofsetinin section içi blok koordinatlarına çevrilmesiyle sınırlıdır. SectionPos origin/center/chunk dönüşümleri, blockToSection ve packed üretim yolları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
