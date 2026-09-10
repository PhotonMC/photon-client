# P01.c — BlockPos betweenClosed AABB sözleşmesi

Bu dilim, BlockPos'ın AABB alan `betweenClosed` overload'unu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:356-360` ve `.data/minecraft/source/net/minecraft/world/phys/AABB.java:22-29` incelendi. AABB constructor'ı uçları eksen bazında normalize eder; BlockPos overload'u normalize edilmiş min/max double değerlerini `Mth.floor` ile integer BlockPos uçlarına çevirip mevcut inclusive range yoluna verir.

## Davranış sözleşmesi

- AABB'nin her eksendeki küçük ve büyük ucu önce Java `Math.min`/`Math.max` semantiğiyle seçilir.
- Min ve max double uçları `Mth.floor` ile bağımsız i32 koordinatlara dönüştürülür.
- Floor sonrası uçlar inclusive olarak ve x hızlı, sonra y, sonra z sırasında taranır.
- Signed zero sonucu koordinat davranışını değiştirmez; negatif kesirler bir önceki integer koordinata iner.
- Bu dilimde yalnızca sonlu, küçük aralıklar kullanılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_closed_aabb"` ve altı f64 bit alanlı `aabb` nesnesini taşır. Sonuç `positions` alanında BlockPos snapshot listesi içerir.

Java oracle `BlockPos.betweenClosed(AABB)` overload'unu doğrudan tüketir. Rust karşılığı `BlockPos::between_closed_aabb` yordamıdır.

## Kapsam dışı

BoundingBox/stream overload'ları, sonsuz veya dev aralıklar, `withinManhattan` ve diğer BlockPos iterator'ları bu dilime dahil değildir.
