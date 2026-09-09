# P01.c — SectionPos asLong overload'ları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos packed long üretiminin BlockPos, koordinat ve instance çağrı yollarıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:215-228` incelendi. Oracle üç Java overload'unu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`asLong(BlockPos)`, BlockPos'un signed x/y/z bileşenlerini alır, her bileşeni dört bit arithmetic shift ile section koordinatına çevirir ve koordinat overload'una aktarır. `asLong(int, int, int)`, x ve z için 22 bit, y için 20 bit alanı maskeler ve bu alanları SectionPos düğümündeki sabit konumlarına yerleştirir. Instance `asLong()`, mevcut section bileşenlerini aynı koordinat yoluna verir.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Overload yolları | 3 | Origin, section sınırlarını geçen pozitif/negatif BlockPos girdileri |
| Packed alan sınırları | 2 | 22/20 bit pozitif ve negatif section sınırları |
| Keyfi ve i32 girdiler | 1 | Keyfi packed BlockPos ile i32 sınırlarında koordinat/instance maskesi |

Girdiler `family: "math"`, `operation: "sectionpos_as_long"`, onaltılık 16 haneli `block_node` ve signed decimal i32 `x`, `y`, `z`, `section_x`, `section_y`, `section_z` alanlarını kullanır. `result.value` içinde `from_block_pos`, `coordinates` ve `instance` unsigned lowercase hexadecimal packed sonuçları bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::as_long_from_block_pos`, `as_long_coordinates` ve `as_long`, daha önce doğrulanan BlockPos unpack, section arithmetic shift ve SectionPos field-mask packing yollarını kullanır. `examples/behavior_adapter.rs` sonuçları JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını karşılaştırma oracle'ı olarak çağırır.
