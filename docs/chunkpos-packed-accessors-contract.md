# P01.c — ChunkPos packed koordinat erişimcileri sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed ChunkPos long içinden signed x ve z koordinatlarının çıkarılmasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:85-91` incelendi. Oracle `ChunkPos.getX(long)` ve `ChunkPos.getZ(long)` metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Packed giriş, 64 bit ham değeri temsil eden 16 haneli küçük harfli unsigned hex metnidir. Düşük 32 bit alan x, yüksek 32 bit alan z olarak okunur; her alan Java `int` dönüşümüyle signed i32 değerine taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Packed alan erişimi | 6 | Sıfır, pozitif ve negatif alanlar, iki signed i32 sınır kombinasyonu ve alternatif bit deseni |

Girdiler `family: "math"`, `operation: "chunkpos_packed_accessors"` ve 16 haneli `packed` alanını kullanır. `result.value.x` ve `result.value.z`, signed decimal string sonuçlarıdır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::get_x` ve `ChunkPos::get_z`, packed değerin iki 32 bit alanını Java ile aynı signed yorumla döndürür. `examples/behavior_adapter.rs` ham packed hex girdisini bu API’ye bağlar; `tests/behavior/java/MathOracle.java` karşılaştırma oracle’ı olarak Java erişimcilerini çağırır.
