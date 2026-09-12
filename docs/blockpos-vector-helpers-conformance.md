# P01.c — BlockPos vector yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, BlockPos'ın `Vec3i` delta alan offset/subtract yolları ile scalar multiply yordamını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:121-139` ve `.data/minecraft/source/net/minecraft/core/Vec3i.java:92-110` incelendi. BlockPos, Vec3i'nin üç koordinatlı değeridir; `offset(Vec3i)` getter sonuçlarını BlockPos'ın int offset yoluna verir. `subtract(Vec3i)` bileşenleri unary negation ile tersleyerek aynı yola bağlar. `multiply(int)` bir ve sıfır ölçeklerini kısa yoldan döndürür.

## Davranış sözleşmesi

- `offset(Vec3i)` delta'nın x/y/z bileşenlerini sırasıyla mevcut koordinatlara ekler; üç bileşen de sıfırsa mevcut değeri korur.
- `subtract(Vec3i)` her delta bileşenini Java int unary negation ile tersleyip offset yolunu kullanır; `Integer.MIN_VALUE` negation sonucu yine `MIN_VALUE` olur.
- `multiply(1)` mevcut değeri, `multiply(0)` `ZERO` değerini döndürür; diğer ölçekler üç koordinatı ayrı ayrı i32 ile çarpar.
- Java int toplama, negation ve çarpma taşmaları i32 sarmasıyla gözlemlenir.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_vector_helpers"`, signed decimal `x`/`y`/`z`, delta için `dx`/`dy`/`dz` ve signed decimal `scale` alanlarını taşır. Sonuç `offset_vec3i`, `subtract_vec3i` ve `multiply` BlockPos snapshot'larını içerir.

Java oracle Vec3i overload'larını gerçek `BlockPos` delta nesnesiyle çağırır. Rust karşılığı `BlockPos::offset_by_block_pos`, `subtract_block_pos` ve `multiply` yordamlarıdır; `BlockPos` argümanı Java Vec3i uyumlu integer delta rolünü üstlenir.

## Kapsam dışı

Vec3i bağımsız codec/equality/mesafe yordamları, üç eksenli `multiply` overload'u, mutable BlockPos ve BlockPos rotate/cross yolları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_vector_helpers` işlemi, BlockPos'ın Vec3i offset/subtract forwarding'ini ve scalar multiply yolunu Java oracle ile karşılaştırır.

Java math capture kimliği `1cf201798a30a0d8cc2392ea50cb2c7c4aa0b5c595d1d2953964c9591129c7d2` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `869668c9ff9a2a713bee0bacb11c3fbfba9740ef43b6dbe378c6b705ad460d81` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca BlockPos vector delta ve scalar multiply yardımcılarını kapsar; Vec3i'nin bağımsız yordamları ayrı dilimlerdedir.
