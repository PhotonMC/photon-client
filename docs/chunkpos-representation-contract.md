# P01.c — ChunkPos temsil ve hash sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos` record'unun Java metin temsilini, instance `hashCode()` sonucunu ve iki signed koordinat alanı için static hash fonksiyonunu taşımaktır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:93-102` içindeki hash yolları ve `world/level/ChunkPos.java:172-175` içindeki `toString()` metodu incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde record equality, packing/codec'ler, region/block yardımcıları, range/distance ve istemciye bağlı chunk davranışları ölçülmez.

## Davranış sözleşmesi

- `toString()`, x ve z signed decimal değerlerini `[x, z]` biçiminde, araya virgül ve boşluk koyarak üretir.
- `hashCode()`, aynı x/z değerlerini static hash yoluna verir.
- Static `hash(x, z)`, x ve z için sabit çarpan/toplama dönüşümlerini XOR'lar.
- Java int çarpma ve toplama taşmaları i32 içinde sarar; XOR signed bit desenini korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Temsil ve hash | 6 | Köken, negatif değerler, signed sınırlar, alternating bit desenleri ve overflow girdileri |

Girdiler `family: "math"`, `operation: "chunkpos_representation"`, `x`, `z`, `hash_x` ve `hash_z` signed decimal i32 metinlerini kullanır. `value` altında Java metin temsili, instance hash ve static hash sonucu decimal string olarak taşınır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::to_string_java`, `hash_code_java` ve `hash` Java'nın temsil biçimini ve wrapping hash aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.
