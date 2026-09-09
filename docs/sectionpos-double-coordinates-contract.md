# P01.c — SectionPos double koordinat yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `SectionPos.posToSectionCoord(double)` ve `SectionPos.blockToSectionCoord(double)` overload'larının kayan nokta girdisini section koordinatına çevirme yoludur.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:77-87` ve `.data/minecraft/source/net/minecraft/util/Mth.java:66-68` incelendi. Oracle, iki Java overload'unu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`posToSectionCoord(double)`, girdiye `Mth.floor` uygular ve sonucu int `blockToSectionCoord` yoluna verir. `blockToSectionCoord(double)` da `Mth.floor` sonucunu dört bit arithmetic shift ile section koordinatına çevirir. Negatif kesirler aşağı yuvarlanır. `Mth.floor` içindeki Java `double` → `int` daraltması NaN için sıfır, aralık dışı değerler için signed int sınırlarını korur; ardından section dönüşümünün `>> 4` davranışı uygulanır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `posToSectionCoord(double)` | 3 | Pozitif sınır, negatif subnormal ve en büyük sonlu double |
| `blockToSectionCoord(double)` | 3 | Pozitif kesir, negatif sınır ve NaN |

Girdiler `family: "math"`, `operation: "sectionpos_double_coordinates"`, `fn` ve 16 haneli unsigned hexadecimal f64 bit desenini taşıyan `value` alanlarını kullanır. `fn`, `pos_to_section` veya `block_to_section` değerlerinden biridir. `result.value`, signed decimal i32 section koordinatıdır.

Bu sözleşme yalnızca iki double overload'unun son int sonucunu kapsar; diğer `Mth.floor` overload'ları ve SectionPos stream/codec yolları ayrı dilimlerdir.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::pos_to_section_coord` ve `block_to_section_coord_f64`, Java `Mth.floor` ile ortak `math::floor_f64` yolunu ve SectionPos'un int section dönüşümünü taşır. `examples/behavior_adapter.rs` f64 bit girdisini bu API'lere bağlar.
