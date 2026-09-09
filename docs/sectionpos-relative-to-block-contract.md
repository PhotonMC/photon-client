# P01.c — SectionPos relative-to-block sözleşmesi

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

`src/binary.rs` içindeki `SectionPos::relative_to_block_x`, `relative_to_block_y`, `relative_to_block_z` ve `relative_to_block_pos`, SectionPos blok origin helper'larını ve short alanlarının unsigned bit yorumunu Java ile aynı sırada kullanır. `examples/behavior_adapter.rs` signed short girdisini u16 bit desenine çevirerek bu API'lere bağlar; `tests/behavior/java/MathOracle.java` Java erişimcilerini karşılaştırma oracle'ı olarak çağırır.
