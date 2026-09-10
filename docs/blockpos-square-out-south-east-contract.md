# P01.c — BlockPos squareOutSouthEast sözleşmesi

Bu dilim, BlockPos'ın deprecated `squareOutSouthEast` yardımcı metodunun sabit dört konumlu sırasını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:260-263` incelendi. Yardımcı, `Stream.of(from, from.south(), from.east(), from.south().east())` çağrısıyla dört BlockPos sonucu üretir.

## Davranış sözleşmesi

- Çıktı sırası kaynak, south, east ve south-east olarak korunur.
- South yalnızca Z'yi bir artırır; east yalnızca X'i bir artırır; Y değişmez.
- Koordinat adımları Java i32 aritmetiğiyle sarmalanır.
- Stream sonuçları capture sırasında bağımsız BlockPos snapshot'ları olarak gözlemlenir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_square_out_south_east"` ve signed decimal `x`/`y`/`z` alanlarını taşır. Sonuç `positions` alanında dört snapshot içerir.

Java oracle `BlockPos.squareOutSouthEast` stream'ini doğrudan tüketir. Rust karşılığı `BlockPos::square_out_south_east` yordamıdır.

## Kapsam dışı

Diğer BlockPos stream/iterator yardımcıları, mutable BlockPos API'si ve AABB overload'ları bu dilime dahil değildir.
