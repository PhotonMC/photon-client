# P01.c — SectionPos spatial yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos'ın blok origin'i, merkez blok konumu ve ChunkPos dönüşümleridir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:202-215` incelendi. Oracle `SectionPos.origin()`, `center()` ve `chunk()` instance metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`origin`, section'ın minimum blok koordinatlarını döndürür. `center`, origin'e her eksende 8 ekleyen BlockPos offset yolunu kullanır. `chunk`, section x/z koordinatlarını ChunkPos'a aktarır; y düşürülür.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Spatial dönüşümler | 6 | Köken, pozitif/negatif section'lar, shift sınırı ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_spatial"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value` içinde `origin`, `center` BlockPos snapshot'ları ve `chunk` ChunkPos snapshot'ı bulunur.

## Rust karşılığı

`crates/voxel-core/src/binary.rs` içindeki `SectionPos::origin`, `center` ve `chunk`, daha önce doğrulanan min blok ve BlockPos offset yollarını kullanarak Java instance sonuçlarını taşır. `examples/behavior_adapter.rs` bu üç sonucu JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java metotlarını karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, SectionPos `origin()`, `center()` ve `chunk()` sonuçlarını Java ile aynı BlockPos ve ChunkPos değerlerine dönüştürür.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos spatial yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos origin/center/chunk dönüşümleriyle sınırlıdır. SectionPos blockToSection, zero node, sectionToChunk packed yolları ve blocksInside/range/aroundAndAtBlockPos akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
