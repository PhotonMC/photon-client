# P01.c — BlockPos betweenClosedStream sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

Rust adapter `blockpos_between_closed_streams` işlemi dört stream overload'unu Java oracle ile karşılaştırır.

Java math capture kimliği `d30d65d9b4bd9942adff6eb2baa1aefa6a01d87168a03ee36ed8855dfe389bad` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt BlockPos, BoundingBox, AABB ve int coordinate stream overload'larının finite küçük girdilerdeki forwarding'ini kapsar.

Birleşik capture kimliği `2bbcd4fb142817a005f14a5179e10209375a77701cd9346de056d1a974b1fcdb` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
