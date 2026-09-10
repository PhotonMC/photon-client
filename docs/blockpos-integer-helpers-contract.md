# P01.c — BlockPos integer yardımcıları sözleşmesi

Bu dilim, Vec3i'den gelen BlockPos integer mesafe metriklerini ve Axis koordinat seçimini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/Vec3i.java:223-242` incelendi. Manhattan yolu int farklarını Java `Math.abs` sonrasında f32'ye yükseltir ve toplamı int'e çevirir; chessboard yolu int mutlak farklarının maksimumunu alır; `get(Axis)` seçilen bileşeni döndürür.

## Davranış sözleşmesi

- `distManhattan(Vec3i)`, `pos - this` x/y/z farklarını Java int aritmetiğiyle hesaplar, her birini `Math.abs` ile işler, f32 toplamı x sonra y sonra z sırasıyla kurar ve Java int cast'iyle sonuçlandırır.
- `distChessboard(Vec3i)`, `this - pos` farklarının Java int mutlak değerlerini alıp x/y/z maksimumunu döndürür.
- `get(Axis)`, X, Y veya Z bileşenlerinden yalnızca seçileni döndürür.
- int çıkarma ve `Math.abs(Integer.MIN_VALUE)` kenarları Java’nın i32 davranışıyla korunur.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_integer_helpers"`, signed decimal x/y/z ve nested `other` BlockPos alanlarını taşır. Sonuç `dist_manhattan`, `dist_chessboard` ve X/Y/Z seçimlerini içeren `get` nesnesini taşır.

Java oracle `BlockPos` üzerinde gerçek `distManhattan`, `distChessboard` ve `get(Direction.Axis)` çağrılarını çalıştırır. Rust karşılığı `BlockPos::dist_manhattan`, `dist_chessboard` ve `get` yordamlarıdır.

## Kapsam dışı

Vec3i f64 mesafe yolları, `closerToCenterThan`, `toMutable`, codec/equality, mutable BlockPos ve aralık/random iterasyonları bu dilime dahil değildir.
