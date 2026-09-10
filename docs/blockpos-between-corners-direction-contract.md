# P01.c — BlockPos between corners in direction sözleşmesi

Bu dilim, yön işaretine göre başlayan ve eksen önceliğine göre ilerleyen BlockPos köşe traversal'ını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:524-616` ve `Direction.java:379-380` incelendi. AABB overload'u normalize edilmiş uçları floor eder; BlockPos ve ham koordinat overload'ları ortak int yoluna yönlenir. Ortak yol, uçları eksen bazında normalize eder, yön bileşenleriyle başlangıç köşesini seçer, X/Z mutlak büyüklüklerinin strict karşılaştırmasıyla Y-X-Z veya Y-Z-X eksen sırasını kurar ve son ekseni en hızlı değiştirir.

## Davranış sözleşmesi

- AABB min/max uçları önce `Mth.floor` ile integer köşelere çevrilir.
- Her overload aynı normalize edilmiş integer koordinat traversal'ını kullanır.
- Direction bileşeni `>= 0.0` ise ilgili eksende minimum köşe ve pozitif adım, aksi halde maksimum köşe ve negatif adım seçilir; signed zero pozitif dalı izler, NaN negatif dalı izler.
- `abs(x) < abs(z)` yalnızca true olduğunda Y-Z-X, diğer tüm sonuçlarda Y-X-Z eksen sırası seçilir.
- Üçüncü eksen en hızlı, ikinci eksen onun bitiminde, birinci eksen en yavaş ilerler; her nokta bağımsız BlockPos snapshot'ı olarak gözlenir.
- Farklar Java i32 çıkarma semantiğiyle hesaplanır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_corners_direction"`, bir AABB, iki BlockPos, iki ham integer köşe ve f64 bit alanlı direction taşır. Sonuç `aabb`, `block_pos` ve `coordinates` dizilerini içerir.

Java oracle üç overload'u ayrı ayrı tüketir. Rust karşılığı `BlockPos::between_corners_in_direction_aabb`, `between_corners_in_direction` ve `between_corners_in_direction_coordinates` yordamlarıdır.

## Kapsam dışı

`MutableBlockPos` mutasyon API'si, breadth-first traversal, dünya collider toplama ve AABB dışındaki iterator kaynakları bu dilime dahil değildir.
