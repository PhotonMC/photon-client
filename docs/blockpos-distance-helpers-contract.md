# P01.c — BlockPos distance yardımcıları sözleşmesi

Bu dilim, Vec3i'den gelen BlockPos mesafe ve eşik yardımcılarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/Vec3i.java:193-221` incelendi. BlockPos, Vec3i mesafe yollarını aynı x/y/z getter'larıyla kullanır. Düşük köşe ve merkez mesafeleri f64 bileşen farklarını Java kaynak sırasıyla kareler; `closerThan` strict `<` karşılaştırması yapar.

## Davranış sözleşmesi

- `distSqr(Vec3i)`, hedefin x/y/z değerlerini `distToLowCornerSqr` yoluna verir.
- `distToLowCornerSqr(x, y, z)`, BlockPos koordinatlarını f64'e yükseltip hedef bileşenlerden çıkarır; kareleri x, sonra y, sonra z sırasıyla toplar.
- `distToCenterSqr(x, y, z)`, her BlockPos bileşenine `0.5` ekleyip hedef noktadan çıkarır ve kareleri aynı sırayla toplar.
- `closerThan(Vec3i, distance)`, `distSqr` sonucunu `distance * distance` ile strict olarak karşılaştırır.
- f64 signed zero, Infinity, NaN ve i32 sınırlarının dönüşüm/aritmetik sonuçları bit düzeyinde korunur.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_distance_helpers"`, signed decimal `x`/`y`/`z`, nested `other` BlockPos, nested `point` f64 bit alanları ve `distance` f64 bit alanını taşır. Sonuç `dist_sqr`, `dist_low_corner_sqr`, `dist_center_sqr` f64 bitleri ile `closer_than` boolean değerini içerir.

Java oracle `BlockPos.distSqr`, `distToLowCornerSqr`, `distToCenterSqr` ve `closerThan` çağrılarını doğrudan çalıştırır. Rust karşılığı `BlockPos::dist_sqr`, `dist_to_low_corner_sqr`, `dist_to_center_sqr` ve `closer_than` yordamlarıdır.

## Kapsam dışı

`closerToCenterThan(Position)`, `distToCenterSqr(Position)`, `distManhattan`, `distChessboard`, Vec3i codec/equality ve mutable BlockPos yolları bu dilime dahil değildir.
