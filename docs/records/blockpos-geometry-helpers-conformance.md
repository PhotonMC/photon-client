# P01.c — BlockPos geometry yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın yatay Rotation, Vec3i cross ve y bileşeni değiştirme yardımcılarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:211-230`, `.data/minecraft/source/net/minecraft/core/Vec3i.java:182-191` ve `.data/minecraft/source/net/minecraft/world/level/block/Rotation.java:15-21` incelendi. Rotation yolları yalnızca x/z bileşenlerini permüte eder; cross üç i32 bileşenli signed işlemden oluşur; `atY` x ve z'yi korur.

## Davranış sözleşmesi

- `rotate(NONE)` mevcut BlockPos'ı korur.
- `rotate(CLOCKWISE_90)` `(x, y, z)` değerini `(-z, y, x)`; `rotate(CLOCKWISE_180)` `(-x, y, -z)`; `rotate(COUNTERCLOCKWISE_90)` `(z, y, -x)` olarak üretir.
- `cross(Vec3i)` `(y·uz - z·uy, z·ux - x·uz, x·uy - y·ux)` işlem sırasını kullanır.
- `atY(y)` x ve z'yi koruyup y'yi verilen int ile değiştirir.
- Java int unary negation, çarpma ve çıkarma taşmaları i32 sarmasıyla gözlemlenir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_geometry_helpers"`, signed decimal `x`/`y`/`z`, `rotation`, cross delta'sı için `up_x`/`up_y`/`up_z` ve signed decimal `at_y` alanlarını taşır. Sonuç `rotate`, `cross` ve `at_y` BlockPos snapshot'larını içerir.

Java oracle gerçek `Rotation` enum'u ile `BlockPos.rotate` çağrısını ve gerçek `BlockPos` delta'sı ile `cross(Vec3i)` overload'unu çalıştırır. Rust karşılığı `BlockPos::rotate`, `cross` ve `at_y` yordamlarıdır.

## Kapsam dışı

Rotation codec/random/Direction yardımcıları, Vec3i'nin bağımsız cross/mesafe yolları, mutable BlockPos ve BlockPos aralık/random iterasyonları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_geometry_helpers` işlemi, BlockPos rotate/cross/atY yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `42a6ce4635c7072299aaed2523c40a5b4590f224094c628af3435e3f6a7c26f4` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `190e7958671c8092dc326700c20b06e16c0ae714617e2b138073197d1c3da510` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos geometry yardımcılarını kapsar; Rotation enum davranışının diğer yolları ayrı dilimlerdedir.
