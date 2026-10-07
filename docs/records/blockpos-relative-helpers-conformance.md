# P01.c — BlockPos relative yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

Rust adapter `blockpos_relative_helpers` işlemi, BlockPos Direction/Axis relative overload'larını ve altı cardinal yönün tek/sayılı yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `28d59732d2db48e98cee214e142821c2f33b77011d49e204e3a91a407d7d8617` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `ea457925c7c014e392cfb2c4499d58a3e1a9777f6a25c83b14f0f045d028dc96` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos relative ve cardinal yardımcılarını kapsar; diğer BlockPos dönüşümleri ayrı dilimlerdedir.
