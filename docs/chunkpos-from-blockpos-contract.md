# P01.c — ChunkPos BlockPos'tan containing/pack sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `BlockPos` için onu içeren `ChunkPos` değerini bulmak ve aynı x/z section koordinatlarını Java'nın packed long biçimine çevirmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:45-47` içindeki `containing(BlockPos)`, `world/level/ChunkPos.java:81-83` içindeki `pack(BlockPos)` ve `core/SectionPos.java:81-83` içindeki int `blockToSectionCoord` yolu incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `BlockPos.pack`, `SectionPos` node packing'i, ChunkPos stream/JSON codec'leri ve chunk yaşam döngüsü ölçülmez. `BlockPos` y değeri her iki ChunkPos yolunda da kullanılmaz.

## Davranış sözleşmesi

- `ChunkPos.containing(BlockPos)`, BlockPos x ve z bileşenlerini 4 bit arithmetic right shift ile section/chunk koordinatlarına çevirir.
- Negatif block koordinatları aşağı doğru section'a düşer; dönüşüm Java signed i32 bit davranışını korur.
- `ChunkPos.pack(BlockPos)`, containing sonucunun x koordinatını packed long'un düşük 32 bitine, z koordinatını yüksek 32 bitine koyar.
- Packed long, oracle çıktısında `Long.toUnsignedString(..., 16)` ile küçük harfli, baştaki sıfırları kaldırılmış unsigned hex metin olarak taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| BlockPos → ChunkPos | 6 | Köken, pozitif/negatif section sınırları, mixed koordinatlar ve signed i32 uçları |

Girdiler `family: "math"`, `operation: "chunkpos_from_blockpos"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `value.containing` altında ChunkPos x/z snapshot'ı, `value.packed` altında Java unsigned hex packed sonucu bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::containing` ve `ChunkPos::pack_block_pos`, mevcut arithmetic section dönüşümü ve ChunkPos packing yollarını birleştirir. `examples/behavior_adapter.rs` JSONL bağını sağlar; `tests/behavior/java/MathOracle.java` Java `ChunkPos.containing` ve `ChunkPos.pack(BlockPos)` çağrılarını doğrudan gözlemler.
