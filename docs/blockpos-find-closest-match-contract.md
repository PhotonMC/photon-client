# P01.c — BlockPos findClosestMatch sözleşmesi

Bu dilim, BlockPos'ın Manhattan taramasında predicate tarafından kabul edilen ilk konumu seçen yardımcı metodunu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:340-350` incelendi. `findClosestMatch`, yatay yarıçapı X ve Z eksenlerine, dikey yarıçapı Y eksenine aktararak `withinManhattan` iterable'ını kaynak sırasıyla tüketir; ilk predicate eşleşmesinde durur.

## Davranış sözleşmesi

- Arama sırası `withinManhattan(start, horizontalRadius, verticalRadius, horizontalRadius)` ile aynıdır.
- İlk eşleşen konum döndürülür; eşleşme yoksa Optional boş kalır.
- Sıfır yarıçap yalnızca başlangıç konumunu aday yapar.
- Origin ve traversal offset'leri Java i32 sarmasıyla değerlendirilir.
- Bu adapter diliminde predicate, adayın verilen target BlockPos'a eşitliği olarak sabitlenmiştir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_find_closest_match"`, `start` ve `target` BlockPos nesneleri ile signed decimal `horizontal_radius`/`vertical_radius` alanlarını taşır. Eşleşmede sonuç `value` alanında BlockPos snapshot içerir; eşleşme yoksa Java'nın null serialization davranışı nedeniyle sonuç nesnesi boştur.

Java oracle eşitlik predicate'iyle `BlockPos.findClosestMatch` çağrısını doğrudan tüketir. Rust karşılığı generic `BlockPos::find_closest_match` yordamıdır.

## Kapsam dışı

Diğer predicate türleri, `withinManhattan` iterator ayrıntıları, breadth-first traversal ve predicate hata/yan etki sırası bu dilime dahil değildir.
