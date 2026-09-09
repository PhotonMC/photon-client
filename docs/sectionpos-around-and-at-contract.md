# P01.c — SectionPos aroundAndAtBlockPos sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `aroundAndAtBlockPos` üç overload'unun verilen block konumu çevresindeki section node callback sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:267-293` incelendi. Oracle coordinates, BlockPos ve packed BlockPos overload'larını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Metot, block koordinatlarının `-1` ve `+1` komşularını section koordinatına çevirir. Üç aralık da aynı section içindeyse tek node çağrılır. Aksi halde sectionX dış, sectionY orta ve sectionZ iç döngüdür; z en hızlı değişir. Block koordinatı `i32` çıkarma/toplama Java wrapping semantiğiyle yürür.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| coordinates | 3 | Interior, origin corner, x section boundary |
| BlockPos overload | 1 | Three-axis boundary |
| packed overload | 1 | All-one packed BlockPos |
| i32 overflow | 1 | `Integer.MAX_VALUE` x endpoint |

Girdiler `family: "math"`, `operation: "sectionpos_around_and_at_block_pos"` kullanır. `mode: "coordinates"` signed decimal i32 `x`, `y`, `z`; `mode: "blockpos"` aynı fields; `mode: "packed"` 16 haneli unsigned hex `block_node` alır. `result.value.section_nodes`, callback sırasındaki packed SectionPos değerlerini unsigned lowercase hex olarak içerir.

Bu sözleşme callback side effect/laziness ve daha geniş streaming API davranışı yerine sonlu küçük node listesiyle sınırlıdır.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::around_and_at_coordinates`, `around_and_at_block_pos` ve `around_and_at_packed`, Java overload yönlendirmesini, i32 wrapping `block±1` hesaplarını ve sectionX/sectionY/sectionZ callback sırasını taşır.
