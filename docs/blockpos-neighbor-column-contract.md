# P01.c — BlockPos neighbor-column sözleşmesi

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
