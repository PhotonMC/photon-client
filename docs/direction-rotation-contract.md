# P01.c — Direction quaternion rotation sözleşmesi

Bu dilim, Java Edition 26.2 `Direction.getRotation()` yolunun sabit yön quaternion'larını ölçer.

## Kaynak ve sınır

`core/Direction.java:144-151` ve pinned JOML 1.10.8 `Quaternionf.rotationX`/`rotationXYZ` yolları incelendi. Java oracle, yön enumunun `getRotation()` sonucunu doğrudan çağırır ve quaternion bileşenlerini raw f32 bitleriyle verir.

## Davranış sözleşmesi

- `up`, identity quaternion `(0, 0, 0, 1)` döndürür.
- `down`, X ekseninde π; `south`, X ekseninde π/2 rotasyonunu kullanır.
- `north`, `west` ve `east`, Java'nın sabit XYZ açı üçlülerini kullanır.
- JOML'nin varsayılan hassas matematik yolu yarım açı sinüsünü, `cosFromSin` işaretini ve f32 bileşen saklamasını belirler.
- Sonuç `x`, `y`, `z`, `w` alanlarında sekiz haneli raw f32 hex değerleri olarak karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_rotation"` ve küçük harfli `name` alanı taşır. Çıktı `value` altında quaternion snapshot'ıdır.

## Kapsam dışı

Genel quaternion işlemleri, matrix dönüşümleri, Direction matrix rotation ve entity bakış yönü bu dilime dahil değildir.
