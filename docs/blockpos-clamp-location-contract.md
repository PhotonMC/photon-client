# P01.c — BlockPos clamp-location sözleşmesi

Bu dilim, BlockPos'ın bir Vec3 konumunu kendi bloğunun iç sınırları içinde tutan yardımcı yolunu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:251-261` ve `.data/minecraft/source/net/minecraft/util/Mth.java:102-108` incelendi. Her eksen için alt sınır `int + 1.0E-5F` f32 toplamasıyla, üst sınır `int + 1.0 - 1.0E-5F` double toplama/çıkarma sırasıyla oluşur. Değer Mth double clamp yoluna verilir.

## Davranış sözleşmesi

- x, y ve z bileşenleri bağımsız olarak clamp edilir.
- Alt sınırda epsilon f32'ye yuvarlanır ve double'a yükseltilir; üst sınırda `1.0` double aritmetiği korunur.
- `Mth.clamp(double)` Java’daki `< min`, NaN ve `Math.min` sırasını izler; alt sınır üst sınırı geçtiğinde de aynı koşullu yol kullanılır.
- Signed zero, Infinity, NaN ve i32 sınırlarının f64 sonuçları bit düzeyinde korunur.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_clamp_location"`, signed decimal `x`/`y`/`z` ve nested `location` f64 bit alanlarını taşır. Sonuç clamp edilmiş Vec3'ü `value` altında f64 bit snapshot'ı olarak içerir.

Java oracle `BlockPos.clampLocationWithin(Vec3)` çağrısını doğrudan çalıştırır. Rust karşılığı `BlockPos::clamp_location_within` yordamıdır.

## Kapsam dışı

BlockPos mutable/immutable dönüşümleri, random/range iterasyonları, AABB box üretimi ve Mth clamp overload'larının bağımsız davranışı bu dilime dahil değildir.
