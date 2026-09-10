# P01.c — BlockPos packed yardımcıları sözleşmesi

Bu dilim, BlockPos packed long üretim, açılım ve offset girişlerini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:47-117` incelendi. X ve Z 26 bitlik signed alanlarda, Y 12 bitlik signed alanda taşınır; `getFlatIndex` packed değerin düşük dört bitini temizler.

## Davranış sözleşmesi

- `PACKED_HORIZONTAL_LENGTH` 26, `PACKED_Y_LENGTH` 12 ve `MAX_HORIZONTAL_COORDINATE` 33554431 değerlerini taşır; `ZERO` `(0, 0, 0)` konumudur.
- `asLong()` ve koordinat overload'u x/y/z alanlarını maskeli packed long'a yerleştirir; `of` ve `getX/Y/Z` alanları signed olarak açar.
- `getFlatIndex` Java long bit maskesiyle düşük dört biti sıfırlar.
- Packed `offset` Direction overload'u yönün üç step getter'ını kullanır; üç adımlı overload Java i32 toplamasıyla alanları günceller ve yeniden pack eder.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_packed_helpers"`, signed decimal `x`/`y`/`z`, 16 haneli unsigned `neighbor_node`, Direction `direction` ve signed decimal `step_x`/`step_y`/`step_z` alanlarını taşır. Sonuç sabitleri, packed/decode/accessor görünümlerini, `getFlatIndex` değerini ve iki packed offset sonucunu içerir.

Rust karşılığı `BlockPos::as_long`, `as_long_coordinates`, `of`, `get_x`, `get_y`, `get_z`, `get_flat_index`, `offset_packed` ve `offset_packed_by` yordamlarıdır.

## Kapsam dışı

BlockPos mutable alt sınıfı, yönlü instance offset zincirleri, geometri aralıkları, random iterasyonlar ve stream codec byte tüketimi bu dilime dahil değildir.
