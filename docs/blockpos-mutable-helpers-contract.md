# P01.c — BlockPos mutable helpers sözleşmesi

Bu dilim, MutableBlockPos'ın Java'daki yerinde güncelleme ve immutable sonuç davranışını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:231-237` ve `618-740` incelendi. BlockPos `immutable()` kendisini, `mutable()` yeni MutableBlockPos'ı üretir. MutableBlockPos integer/double constructor'ları ve `set` overload'ları koordinatları yazar; double yolları `Mth.floor`, packed yol `BlockPos.getX/Y/Z` kullanır. `setWithOffset`, `move`, `clamp` ve setX/Y/Z durumu değiştirir. Mutable override'ları `offset`, `multiply`, `relative` ve `rotate` için mutasyonu koruyup yeni immutable BlockPos sonucu döndürür.

## Davranış sözleşmesi

- Mutable state her mutation'dan sonra yeni koordinatları taşır ve method zincirleri aynı mutable nesneyi hedefler.
- Double constructor/set floor dönüşümünü Java ile aynı yapar; packed set signed alanları BlockPos packed düzeninden çıkarır.
- Direction ve integer offset/move yolları Java i32 toplama ve çarpma sarmasını korur.
- `clamp` yalnızca seçilen ekseni `Mth.clamp(int)` ile sınırlar.
- `offset`/`multiply`/`relative`/`rotate` override'ları mutable state'i değiştirmeden immutable dönüş değeri verir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_mutable_helpers"`, bir initial BlockPos, f64 bit alanlı double constructor değerleri ve sıralı mutation adımları taşır. Sonuç constructor gözlemleri ile her adımın dönen değerini ve mutable state snapshot'ını içerir.

Rust karşılığı `BlockPos::immutable`, `BlockPos::mutable`, `MutableBlockPos` constructor/set/move/clamp ve immutable-result yordamlarıdır.

## Kapsam dışı

`AxisCycle` set overload'u, gerçek subclass polymorphism'i, codec'ler ve dünya traversal'ı bu dilime dahil değildir.
