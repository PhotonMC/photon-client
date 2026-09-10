# P01.c — BlockPos relative yardımcıları sözleşmesi

Bu dilim, BlockPos'ın Direction/Axis relative overload'ları ile cardinal yön yardımcılarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:121-209` incelendi. Direction relative yolu yönün step bileşenlerini adımla çarpar; Axis yolu yalnızca seçilen eksene adım ekler. Cardinal yardımcıları bu yolların sabit yön eşlemeleridir.

## Davranış sözleşmesi

- `relative(Direction)` bir step ekler; `relative(Direction, int)` her step bileşenini verilen i32 adımla çarpar.
- `relative(Axis, int)` yalnızca X, Y veya Z bileşenini değiştirir.
- `above`, `below`, `north`, `south`, `west` ve `east` sırasıyla UP, DOWN, NORTH, SOUTH, WEST ve EAST yönlerine birer adım uygular.
- Cardinal sayılı overload'lar aynı yönü verilen i32 adımla kullanır; sıfır adım mevcut koordinat değerini korur.
- Çarpma ve toplama Java i32 sarmasıyla gözlemlenir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_relative_helpers"`, signed decimal `x`/`y`/`z`, Direction `direction`, Axis `axis` ve signed decimal `steps` alanlarını taşır. Sonuç seçilen Direction/Axis relative değerlerini ve tüm cardinal tek/sayılı yardımcılarını BlockPos snapshot'ları olarak içerir.

Rust karşılığı `BlockPos::relative_direction`, `relative_axis` ve cardinal yardımcılarının `above`, `above_steps`, `below`, `below_steps`, `north`, `north_steps`, `south`, `south_steps`, `west`, `west_steps`, `east`, `east_steps` yordamlarıdır.

## Kapsam dışı

`offset(Vec3i)`, `subtract`, `multiply`, rotate/cross, mutable BlockPos ve BlockPos aralık/random iterasyonları bu dilime dahil değildir.
