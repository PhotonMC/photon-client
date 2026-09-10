# P01.c — BlockPos between-closed sözleşmesi

Bu dilim, BlockPos'ın iki BlockPos ucu ve altı int koordinatı alan inclusive `betweenClosed` yollarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:356-418` incelendi. BlockPos overload'u her eksende min/max seçerek koordinat overload'una yönlenir. Koordinat overload'u `width`, `height`, `depth` ve `end` değerlerini Java int aritmetiğiyle kurar; cursor x'i hızlı, sonra y, sonra z ilerler.

## Davranış sözleşmesi

- BlockPos uçları her eksende bağımsız normalize edilir ve inclusive olarak dahil edilir.
- Koordinat overload'unda index x ekseninde, `index / width` içindeki slice y ekseninde, sonraki bölüm z ekseninde tüketilir.
- Her sonuç BlockPos snapshot'ı olarak gözlemlenir; Java mutable cursor'ın önceki sonuçları değiştirmemesi capture sırasında sabitlenir.
- Genişlik, yükseklik, derinlik ve koordinat toplamlarındaki int aritmetiği Java i32 sarmasıyla ele alınır; bu dilimde sonlu küçük aralıklar kullanılır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_closed"`, nested `from`/`to` BlockPos uçları ve signed decimal `min_x`/`min_y`/`min_z`/`max_x`/`max_y`/`max_z` alanlarını taşır. Sonuç `from_to` ve `coordinates` listelerini içerir.

Java oracle iki `BlockPos.betweenClosed` overload'unu doğrudan tüketir. Rust karşılığı `BlockPos::between_closed` ve `between_closed_coordinates` yordamlarıdır.

## Kapsam dışı

`betweenClosed(AABB)`, stream/spliterator ayrıntıları, mutable BlockPos dönüşümleri, `withinManhattan`, random iterasyonlar ve `neighborColumn` bu dilime dahil değildir.
