# P01.c — SectionPos offset overload'ları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos düğümünü Direction veya üç i32 adımla kaydıran static yollar ile SectionPos instance kaydırma yoludur.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:69-75,230-232` incelendi. Oracle static Direction, static üç adım ve instance `offset` yollarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`offset(long, Direction)`, yönün x/y/z step değerlerini üç adımlı static overload'a verir. Bu yol packed SectionPos alanlarını signed olarak açar, adımları Java i32 toplamasıyla ekler ve sonucu `asLong` alan maskeleriyle yeniden pack eder. Instance `offset(int, int, int)`, üç adımın tamamı sıfırsa mevcut SectionPos'u korur; aksi halde bileşenleri aynı Java i32 wrapping toplamasıyla yeni SectionPos'a taşır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Overload yolları | 3 | Sıfır, pozitif ve negatif packed/instance kaydırmalar; Direction adımları |
| Packed alan sınırı | 1 | Pozitif x/y/z alan sınırında maskelenen static sonuç |
| i32 taşması | 1 | Packed static toplama ve instance bileşen taşması |
| Keyfi zero yolu | 1 | Keyfi packed node ve sıfır instance adımı |

Girdiler `family: "math"`, `operation: "sectionpos_offset"`, 16 haneli unsigned hex `section_node`, küçük harfli `direction`, signed decimal i32 `step_x`, `step_y`, `step_z`, `x`, `y`, `z` alanlarını kullanır. `result.value` içinde Direction static yolu `packed_direction`, üç adımlı static yol `packed_steps`, instance bileşenleri ve instance packed sonucu bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::offset_packed`, `offset_packed_by` ve `offset`, daha önce doğrulanan Direction step, signed SectionPos unpack/pack ve i32 wrapping yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını karşılaştırma oracle'ı olarak çağırır.
