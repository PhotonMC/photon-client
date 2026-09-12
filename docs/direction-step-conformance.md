# P01.c — Direction step vektörü sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `Direction.getStepX/Y/Z()` getter'ları ile `Direction.step()` vektör yolunu ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/Direction.java:255-273` incelendi. Üç getter, Direction normalinin i32 bileşenlerini verir; `step()` ise aynı normalin f32 `Vector3f` kopyasını döndürür.

## Davranış sözleşmesi

- DOWN, UP, NORTH, SOUTH, WEST ve EAST için i32 adım bileşenleri sırasıyla `(0,-1,0)`, `(0,1,0)`, `(0,0,-1)`, `(0,0,1)`, `(-1,0,0)` ve `(1,0,0)` değerleridir.
- `step()` aynı değerleri f32 bileşenler olarak taşır; sıfır bileşenleri pozitif zero'dur.
- Getter'lar ve `step()` yönün saklanan normalini değiştirmez.

## JSONL arayüzü

İstek `family: "math"`, `operation: "direction_step"` ve küçük harfli yön `name` alanını taşır. Sonuç `step_x`, `step_y`, `step_z` i32 alanları ile `step` altında x/y/z raw f32 bitlerini içerir.

Rust karşılığı `Direction::get_step_x`, `get_step_y`, `get_step_z` ve `step_vec3f` yordamlarıdır.

## Kapsam dışı

`getUnitVec3i/getUnitVec3/getUnitVec3f`, `Vec3.relative` ve Direction'ın diğer dönüşüm veya entity bağımlı yolları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `direction_step` işlemi, Direction’ın i32 adım getter'larını ve f32 step vektörünü Java `getStepX/Y/Z` ve `step()` çağrılarıyla karşılaştırır.

Java math capture kimliği `0223f82bc89c86357ba4b5147e4713b6b2e463f1872919bb21cd7076306c66dc` olarak `.data/captures/behavior/` altında tutulur. Java capture iki ayrı JVM çalıştırmasında kararlı bulundu ve debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `c5794094082a1506652a7f250204d50d0871bae63d80945997938db764059d33` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca Direction step getter ve vektör yüzlerini kapsar; unit vector getter'ları ve diğer Direction davranışları ayrı dilimlerdedir.
