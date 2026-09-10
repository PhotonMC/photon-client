# P01.c — BlockPos random range sözleşmesi

Bu dilim, BlockPos'ın `randomBetweenClosed` ve `randomInCube` random iterator'larını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:247-285` incelendi. `randomInCube`, merkezden boyut çıkarıp ekleyerek altı sınırı kurar ve `randomBetweenClosed` metoduna yönlenir. Random iterator her elemanda x, sonra y, sonra z için birer bounded random çekimi yapar; limit sıfır veya negatifse çekim yapmadan biter.

## Davranış sözleşmesi

- Her konum için random draw sırası x, y, z olarak korunur.
- Her eksen sonucu ilgili inclusive aralığın alt sınırından başlayarak `nextInt(width)` sonucu ile Java i32 toplamasıyla kurulur.
- `randomInCube` sınırları merkez ± boyut işlemlerinin Java i32 sarmasıyla hesaplanır.
- Limit `<= 0` olduğunda iterator boş kalır ve random kaynağı tüketilmez.
- Mutable cursor çıktıları capture sırasında bağımsız BlockPos snapshot'larına çevrilir; sonrasındaki `nextInt` değerleri tüketim sırasını görünür kılar.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_random_ranges"`, signed decimal seed/limit/altı sınır, `center` BlockPos ve `size` alanlarını taşır. Sonuç `between`, `after_between`, `cube` ve `after_cube` alanlarını içerir.

Java oracle iki overload'u aynı seed ile ayrı random kaynaklarda tüketir. Rust karşılığı `BlockPos::random_between_closed` ve `BlockPos::random_in_cube` yordamlarıdır.

## Kapsam dışı

`withinManhattan`, `spiralAround`, random kaynak implementasyonunun diğer overload'ları ve invalid bound exception metinleri bu dilime dahil değildir.
