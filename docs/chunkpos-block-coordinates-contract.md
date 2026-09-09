# P01.c — ChunkPos blok koordinatı yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir ChunkPos'un kapsadığı blok x/z aralığının minimum, orta ve maksimum uçlarını ve verilen offset ile hesaplanan blok koordinatlarını taşımaktır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:104-126` içindeki `getMiddleBlockX/Z`, `getMinBlockX/Z`, `getMaxBlockX/Z` ve `getBlockX/Z` metotları ile `core/SectionPos.java:128-134` içindeki section-to-block dönüşümü incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `ChunkPos`'un region, BlockPos üretimi, contains, distance, range ve diğer yardımcıları ölçülmez. Blok koordinatı girdileri signed decimal i32 metinlerinden oluşturulur.

## Davranış sözleşmesi

- `getMinBlockX/Z`, chunk koordinatını 4 bit sola kaydırarak kapsanan blok aralığının alt ucunu üretir.
- `getMiddleBlockX/Z`, aynı dönüşümde offset 8'i kullanır.
- `getMaxBlockX/Z`, aynı dönüşümde offset 15'i kullanır.
- `getBlockX/Z(offset)`, section-to-block dönüşümüne verilen offset'i ekler.
- Java int sol kaydırma ve toplama taşmaları i32 içinde sarar; negatif chunk koordinatları taban bölme yerine bit kaydırma semantiğini korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Blok koordinatı yardımcıları | 6 | Köken, pozitif/negatif chunk, aralık sınırı, shift taşması ve offset taşması |

Girdiler `family: "math"`, `operation: "chunkpos_block_coordinates"`, `x`, `z` ve `offset` alanlarını kullanır. Tüm sonuçlar `value` altında minimum, orta, maksimum ve offset'li x/z koordinatlarını signed decimal string olarak taşır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::get_block_x`, `get_block_z`, `get_min_block_x`, `get_min_block_z`, `get_middle_block_x`, `get_middle_block_z`, `get_max_block_x` ve `get_max_block_z` Java'nın 4 bit sola kaydırma ve i32 wrapping add yolunu korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.
