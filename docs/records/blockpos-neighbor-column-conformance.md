# P01.c — BlockPos neighbor-column sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın sabit yatay komşu kolonları boyunca inclusive Y traversal'ını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:420-443` incelendi. `neighborColumn` beş `Vec3i` adımını merkez, north, east, south ve west sırasına koyar. Her adım grubu aynı yatay koordinatlarda başlangıçtan bitişe yürür; `endY > startY` yalnızca Y yönünü seçer, diğer durumda yön `-1` olur.

## Davranış sözleşmesi

- Beş yatay grup sırası `merkez, north, east, south, west` olarak korunur.
- Her grup başlangıç Y'sini ve bitiş Y'sini inclusive içerir.
- Bitiş Y'si başlangıçtan büyükse Y birer artar; eşit veya küçükse birer azalır.
- Yatay ve dikey koordinat toplamları Java i32 sarmasıyla gözlemlenir.
- Java mutable cursor çıktıları capture sırasında bağımsız BlockPos snapshot'larına çevrilir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_neighbor_column"` ve signed decimal `start_x`/`start_y`/`start_z`/`end_y` alanlarını taşır. Sonuç `positions` alanında snapshot listesi içerir.

Java oracle `BlockPos.neighborColumn` iterable'ını doğrudan tüketir. Rust karşılığı `BlockPos::neighbor_column` yordamıdır.

## Kapsam dışı

`spiralAround`, `withinManhattan`, `betweenCornersInDirection`, breadth-first traversal, mutable BlockPos API'si ve AABB/stream overload'ları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_neighbor_column` işlemi, beş yatay komşu grubunun inclusive Y traversal'ını Java oracle ile karşılaştırır.

Java math capture kimliği `e1be58bca503cae554c820096f196717da1af36c4d97f01a421eacd356df15f0` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `neighborColumn` finite listelerini, yatay grup sırasını ve artan/azalan Y traversal'ını kapsar; mutable API ve diğer BlockPos iterator'ları ayrı dilimlerdedir.

Birleşik capture kimliği `394ac801d0d22561c843054f24fad37fb75bc951c4c5f99f370250076c9b383d` ile debug ve release adapter karşılaştırmaları da fark üretmedi.
