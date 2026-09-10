# P01.c — BlockPos betweenClosedStream sözleşmesi

Bu dilim, BlockPos'ın BlockPos, BoundingBox, AABB ve int koordinat alan `betweenClosedStream` overload'larını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:373-394` incelendi. Overload'lar sequential stream'e sarılmış aynı inclusive range traversal'ına yönlenir. BoundingBox uçları eksen bazında normalize edilir; AABB uçları `Mth.floor` ile BlockPos koordinatlarına çevrilir.

## Davranış sözleşmesi

- Dört overload aynı x hızlı, sonra y, sonra z traversal sırasını taşır.
- BlockPos uçları ve BoundingBox uçları componentwise normalize edilir.
- AABB uçları floor sonrası inclusive aralığa dahil edilir.
- Stream tüketimi sonucu mutable cursor değerleri bağımsız BlockPos snapshot'ları olarak gözlemlenir.
- Koordinat arithmetic'i Java i32 sarmasıyla değerlendirilir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_closed_streams"`, BlockPos `from`/`to`, integer `bounding_box`, f64 bit alanlı `aabb` ve altı signed decimal koordinat alanını taşır. Sonuç `block`, `bounding_box`, `aabb` ve `coordinates` listelerini içerir.

Java oracle dört `betweenClosedStream` overload'unu doğrudan tüketir. Rust karşılığı `BlockPos::between_closed_stream`, `between_closed_stream_bounding_box`, `between_closed_stream_aabb` ve `between_closed_stream_coordinates` yordamlarıdır.

## Kapsam dışı

Stream spliterator özellikleri, sonsuz/dev aralıklar, `betweenClosed(AABB)` iterable'ının yeni geometrisi ve diğer BlockPos iterator'ları bu dilime dahil değildir.
