# P01.c — ChunkPos rangeClosed sözleşmesi

Bu dilimde **8 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos.rangeClosed` center/radius ve from/to overload'larının sınırlı, sonlu aralık çıktısıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:203-237` incelendi. Oracle Java stream'ini doğrudan tüketir; beklenen sonuçlar Rust çıktısından türetilmez.

- Center/radius overload'u Java signed i32 çıkarma ve toplama ile from/to uçlarını kurar.
- From/to uçları dahil edilir.
- X ekseni hızlı değişir; X tamamlanınca satırın başlangıç X'i kullanılarak Z bir yönde ilerler.
- Her eksenin yönü, from koordinatı to koordinatından küçükse `+1`, aksi halde `-1` olur. Eşit eksenlerde bu nedenle ilerleme yapılmaz.
- Negatif radius, endpoint aritmetiğinden sonra oluşan aralığın azalan yönde tüketilmesini korur.

Bu dilim devasa overflow aralıklarını, spliterator estimate değerini, stream tembelliğini, paralel stream davranışını, consumer short-circuit davranışını ve chunk yaşam döngüsünü ölçmez.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| from/to | 5 | Tek nokta, artan/azalan dikdörtgen, sabit X ve sabit Z |
| center/radius | 3 | Sıfır, pozitif ve negatif radius |

Girdiler `family: "math"`, `operation: "chunkpos_range_closed"` ve `mode: "from-to"` veya `mode: "center-radius"` alanlarını kullanır. Koordinatlar signed decimal i32 metinleridir. `from-to` için `from_x`, `from_z`, `to_x`, `to_z`; center/radius için `x`, `z`, `radius` alanları taşınır. `result.value.positions`, her ChunkPos'u decimal string `x` ve `z` alanlarıyla sırayla taşır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::range_closed` ve `range_closed_from_to`, endpoint aritmetiğini ve Java'nın x-fastest/z-next traversal sırasını üretir. `examples/behavior_adapter.rs` iki overload'u JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream çıktısını gözlemler.
