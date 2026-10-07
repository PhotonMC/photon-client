# P01.c — SectionPos relative-to-block sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, section içindeki packed short ofsetini blok koordinatlarına ve BlockPos'a çevirmektir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:100-126` incelendi. Oracle `SectionPos.of(int, int, int)` ile SectionPos oluşturur; `sectionRelativeX/Y/Z(short)` ile ofset alanlarını okur ve `relativeToBlockX/Y/Z(short)` ile `relativeToBlockPos(short)` instance metotlarını doğrudan çağırır. Beklenen sonuçlar Rust çıktısından türetilmez.

Relative short değeri Java'nın signed 16 bit taşıyıcısıdır; x, y ve z ofsetleri sırasıyla `>>> 8`, `>>> 0` ve `>>> 4` sonrasında düşük dört bit maskesiyle alınır. Her ofset SectionPos minimum blok koordinatına Java i32 toplamasıyla eklenir. Ham bit deseni Rust adapter'ında u16 olarak korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Relative blok dönüşümü | 6 | Sıfır ve köşe ofsetleri, negatif short değerleri, karışık alanlar ve i32 koordinat taşması |

Girdiler `family: "math"`, `operation: "sectionpos_relative_to_block"`, signed decimal i32 `x`, `y`, `z` ve signed decimal short `relative` alanlarını kullanır. `result.value` relative x/y/z ofsetlerini, üç blok koordinatını ve `block_position` snapshot'ını taşır.

## Rust karşılığı

`crates/voxel-core/src/binary.rs` içindeki `SectionPos::relative_to_block_x`, `relative_to_block_y`, `relative_to_block_z` ve `relative_to_block_pos`, SectionPos blok origin helper'larını ve short alanlarının unsigned bit yorumunu Java ile aynı sırada kullanır. `examples/behavior_adapter.rs` signed short girdisini u16 bit desenine çevirerek bu API'lere bağlar; `tests/behavior/java/MathOracle.java` Java erişimcilerini karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, SectionPos içindeki short relative ofsetini Java `sectionRelativeX/Y/Z` ve `relativeToBlockX/Y/Z` yollarıyla aynı sonuçlara çevirir; `relativeToBlockPos` da aynı üç bileşeni BlockPos'a taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos relative-to-block yardımcıları için 6 özgün senaryo hazırlandı.
- Sıfır/köşe ofsetleri, signed short bit desenleri, karışık alanlar ve i32 koordinat taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `2e1ed7c23719ce14a76b19ea36b422abdc2629c325d726e2e6dc4a0b2542924d`, release `8bbbe1c0f9dd455565fc7c1cc8884bf41ae0c56ee1b3a6a8e91a1facb40a230b`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos short relative ofsetinin section içi blok koordinatlarına çevrilmesiyle sınırlıdır. SectionPos origin/center/chunk dönüşümleri, blockToSection ve packed üretim yolları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
