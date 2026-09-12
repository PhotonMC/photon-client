# P01.c — BlockPos coordinate helpers sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın `containing` ve componentwise `min/max` yardımcılarını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:90-103` incelendi. `containing(double, double, double)` her bileşeni `Mth.floor` ile integer koordinata çevirir; Position overload'u aynı yolu Position bileşenleriyle tüketir. `min` ve `max` x/y/z eksenlerini bağımsız karşılaştırır.

## Davranış sözleşmesi

- Double floor sonucu Java `Mth.floor` ile aynı i32 değerdir; negatif kesirler bir alt integer koordinata iner.
- Vec3 Position forwarding'i x, y, z sırasını korur.
- `min` ve `max` eksenleri birbirinden bağımsız seçer, bütün BlockPos'ı tek değer gibi sıralamaz.
- Signed zero floor sonucunda sıfır koordinatına dönüşür.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_coordinate_helpers"`, f64 bit alanlı `x`/`y`/`z` ve `point`, ayrıca `first`/`second` BlockPos nesnelerini taşır. Sonuç `containing`, `containing_position`, `min` ve `max` snapshot'larını içerir.

Java oracle dört BlockPos helper yolunu doğrudan tüketir. Rust karşılığı `BlockPos::containing`, `containing_vec3`, `min` ve `max` yordamlarıdır.

## Kapsam dışı

Codec/stream codec yolları, AABB floor forwarding'i, mutable BlockPos ve iterator yardımcıları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_coordinate_helpers` işlemi, containing ve componentwise min/max yollarını Java oracle ile karşılaştırır.

Java math capture kimliği `cf1f618be0f377747e5a836bbc7e26ef4aaa766fa6e8a860a8051d98862bbff6` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt sonlu double floor, Vec3 Position forwarding ve componentwise integer min/max yollarını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `f49fd734c2cfc969856e3c0a90e314629b2e517e32cbd1ed1b8f5f55658853fc`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
