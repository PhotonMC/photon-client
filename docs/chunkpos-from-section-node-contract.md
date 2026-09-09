# P01.c — ChunkPos section node dönüşümü sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, packed SectionPos long değerinden ChunkPos packed long üretme yoludur.

## Kaynak ve giriş biçimi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:77-79` ve `core/SectionPos.java:136-145` yolları incelendi. Oracle, `ChunkPos.fromSectionNode(long)` metodunu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Her `section_node` girdisi, SectionPos packed long’unun ham bitlerini taşıyan 16 haneli küçük harfli unsigned hex metnidir. SectionPos x/z alanları 22 bit signed değerlerdir; y alanı 20 bittir.

## Davranış sözleşmesi

- Java, section node içinden SectionPos x ve z alanlarını signed 22 bit olarak çıkarır.
- Section y alanı dönüşümde kullanılmaz.
- Çıkarılan x düşük, z yüksek 32 bit alanlara yerleştirilerek ChunkPos packed long’u üretilir.
- Section x/z uçlarındaki işaret genişletme ve ChunkPos packed long’un unsigned hex gösterimi korunur.

Bu dilimde SectionPos üreticileri, section node offset işlemleri, BlockPos dönüşümleri ve `ChunkPos.rangeClosed` ölçülmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos from section node | 6 | Köken, pozitif koordinatlar, tümü negatif koordinatlar, signed 22 bit minimum/maksimum uçları ve y alanının atıldığı packed node |

Girdiler `family: "math"`, `operation: "chunkpos_from_section_node"` ve `section_node` alanını kullanır. `result.value`, Java long sonucunun unsigned lowercase hex metnidir.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::from_section_node`, mevcut `SectionPos` packed long çözümünden x/z değerlerini alıp ChunkPos `pack` yoluna verir. `examples/behavior_adapter.rs` sonucu JSONL’ye bağlar; `tests/behavior/java/MathOracle.java` Java metodunu doğrudan gözlemler.
