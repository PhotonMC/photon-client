# P01.c — SectionPos packed dönüşümleri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed BlockPos ve SectionPos düğümleri arasındaki SectionPos dönüşümleridir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:117-126` incelendi. Oracle `blockToSection`, `getZeroNode` ve `sectionToChunk` statik yollarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`blockToSection`, packed BlockPos içindeki signed x/y/z alanlarını çıkarır, her ekseni dört bit arithmetic shift ile section koordinatına çevirir ve SectionPos packed düzeninde yeniden birleştirir. Packed `getZeroNode`, section düğümünün düşük 20 bitlik y alanını temizler. Koordinat overload'u x/z section koordinatlarını y=0 ile pack eder ve aynı temizleme yolunu kullanır. `sectionToChunk`, packed SectionPos x/z alanlarını packed ChunkPos'un düşük/yüksek 32 bit alanlarına taşır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Packed dönüşümler | 6 | Origin, pozitif/negatif alanlar, packed sınırlar ve keyfi bit desenleri |

Girdiler `family: "math"`, `operation: "sectionpos_packed_transformations"`, onaltılık 16 haneli `block_node`/`section_node` ve signed decimal i32 `x`/`z` alanlarını kullanır. `result.value` içinde `block_to_section`, packed `zero_node`, koordinat overload'undan `zero_node_coordinates` ve `section_to_chunk` unsigned hexadecimal string sonuçları bulunur.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::block_to_section`, `get_zero_node`, `get_zero_node_from_coordinates` ve `section_to_chunk`, daha önce doğrulanan BlockPos/SectionPos bit alanı yardımcılarını ve Java i32 arithmetic shift yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java statik metotlarını karşılaştırma oracle'ı olarak çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong`, `offset` ve range yardımcıları dışındaki node akışları ile `blocksInside`/`aroundAndAtBlockPos` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
