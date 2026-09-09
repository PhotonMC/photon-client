# P01.c — SectionPos packed dönüşümleri sözleşmesi

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

`src/binary.rs` içindeki `SectionPos::block_to_section`, `get_zero_node`, `get_zero_node_from_coordinates` ve `section_to_chunk`, daha önce doğrulanan BlockPos/SectionPos bit alanı yardımcılarını ve Java i32 arithmetic shift yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java statik metotlarını karşılaştırma oracle'ı olarak çağırır.
