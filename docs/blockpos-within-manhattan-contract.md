# P01.c — BlockPos withinManhattan sözleşmesi

Bu dilim, BlockPos'ın Manhattan reach iterator'ını ve sequential stream forwarding'ini ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:287-354` incelendi. Iterator toplam reach'i maksimum derinlik olarak kullanır; her derinlikte x ve y eksenlerini tarar, kalan z değerini hesaplar ve reach içinde kalan sıfır dışı z değerinin mirror'ını hemen sonraki sonuç olarak üretir. `withinManhattanStream` bu iterable'ı sequential stream'e verir.

## Davranış sözleşmesi

- Her sonuç `abs(x-originX) + abs(y-originY) + abs(z-originZ)` reach sınırları içinde Java kaynak traversal sırasıyla üretilir.
- x, y ve derinlik döngüsü sırası ile sıfır dışı z ve mirror z çifti korunur.
- Eksen reach'leri ve origin offset'leri Java i32 aritmetiğiyle değerlendirilir.
- Mutable cursor sonuçları capture sırasında bağımsız BlockPos snapshot'larına dönüştürülür.
- Stream çıktısı iterable çıktısıyla aynı sırayı taşır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_within_manhattan"`, `origin` BlockPos nesnesi ve signed decimal `reach_x`/`reach_y`/`reach_z` alanlarını taşır. Sonuç `positions` ve `stream_positions` listelerini içerir.

Java oracle hem `BlockPos.withinManhattan` hem `withinManhattanStream` yollarını doğrudan tüketir. Rust karşılığı `BlockPos::within_manhattan` yordamıdır.

## Kapsam dışı

`findClosestMatch`, predicate callback'leri, breadth-first traversal, `spiralAround` ve negatif/taşmalı dev reach'lerin iterator yaşam süresi bu dilime dahil değildir.
