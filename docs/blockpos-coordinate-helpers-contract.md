# P01.c — BlockPos coordinate helpers sözleşmesi

Bu dilim, BlockPos'ın `containing` ve componentwise `min/max` yardımcılarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:90-103` incelendi. `containing(double, double, double)` her bileşeni `Mth.floor` ile integer koordinata çevirir; Position overload'u aynı yolu Position bileşenleriyle tüketir. `min` ve `max` x/y/z eksenlerini bağımsız karşılaştırır.

## Davranış sözleşmesi

- Double floor sonucu Java `Mth.floor` ile aynı i32 değerdir; negatif kesirler bir alt integer koordinata iner.
- Vec3 Position forwarding'i x, y, z sırasını korur.
- `min` ve `max` eksenleri birbirinden bağımsız seçer, bütün BlockPos'ı tek değer gibi sıralamaz.
- Signed zero floor sonucunda sıfır koordinatına dönüşür.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_coordinate_helpers"`, f64 bit alanlı `x`/`y`/`z` ve `point`, ayrıca `first`/`second` BlockPos nesnelerini taşır. Sonuç `containing`, `containing_position`, `min` ve `max` snapshot'larını içerir.

Java oracle dört BlockPos helper yolunu doğrudan tüketir. Rust karşılığı `BlockPos::containing`, `containing_vec3`, `min` ve `max` yordamlarıdır.

## Kapsam dışı

Codec/stream codec yolları, AABB floor forwarding'i, mutable BlockPos ve iterator yardımcıları bu dilime dahil değildir.
