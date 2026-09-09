# P01.c — ChunkPos BlockPos yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `ChunkPos` içindeki offset'lerden `BlockPos` üretmek, orta blok konumunu ve dünya başlangıç konumunu almak ve verilen bir `BlockPos`'un chunk'ın yatay sınırları içinde olup olmadığını kontrol etmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:152-190` içindeki `getBlockAt`, `getMiddleBlockPosition`, `contains` ve `getWorldPosition` metotları incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `BlockPos` packing'i, chunk region/stream/JSON codec'leri, block koordinatı yardımcılarının kendisi, range/distance ve istemciye bağlı chunk davranışları ölçülmez. `contains`, Java'daki gibi yalnızca x/z eksenlerini karşılaştırır; query y değeri sonucu etkilemez.

## Davranış sözleşmesi

- `getBlockAt(x, y, z)`, chunk'ın block x/z dönüşümüne verilen offset'leri uygular ve y değerini doğrudan taşır.
- `getMiddleBlockPosition(y)`, block x/z için 8 offset'ini ve verilen y değerini kullanır.
- `getWorldPosition()`, chunk'ın minimum block x/z koordinatlarını y=0 ile birlikte döndürür.
- `contains(BlockPos)`, minimum ve maksimum block x/z uçlarını kapsayıcı karşılaştırır; y eksenini yok sayar.
- Block koordinatı dönüşümü ve offset toplaması Java i32 sarma davranışını korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos → BlockPos yardımcıları | 6 | Köken, pozitif/negatif chunk, sınır, offset taşması ve shift taşması |

Girdiler `family: "math"`, `operation: "chunkpos_block_positions"`, chunk `x`/`z`, block offset'leri, y ve `contains` query listesini kullanır. `value` altında `block_at`, `middle_block_position`, `world_position` ve sıralı containment sonuçları taşınır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::get_block_at`, `get_middle_block_position`, `get_world_position` ve `contains` Java'nın BlockPos üretim sırasını, inclusive yatay sınırlarını ve i32 wrapping aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.
