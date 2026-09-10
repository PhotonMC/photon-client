# P01.c — Direction step vektörü sözleşmesi

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
