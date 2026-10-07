# P01.c — Direction quaternion rotation sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `Direction::get_rotation`, Java `Direction.getRotation()` için yön başına sabit JOML quaternion yolunu kullanır. JOML'nin yarım açı, `cosFromSin` işareti ve f32 saklama davranışı `crates/voxel-core/src/math.rs` içindeki yardımcılarla taşınır. Adapter sonucu x/y/z/w raw f32 bitleri olarak dışarı verir; Java oracle aynı enum metodunu doğrudan çağırır.

Java math capture kimliği `e37b4611c76826a0b1e25c93a366285d488a856a012e4a9c65998d9e140f9697` olarak `.data/` altında tutulur. Capture iki ayrı JVM çalıştırmasında kararlı bulundu ve Rust debug adapter karşılaştırması fark üretmeden tamamlandı. Birleşik capture kimliği `ddbcf0f3b1df30103517064c4eb9c6bfef9a057d1b10a4f86f786830300869cc` ile debug ve release adapter karşılaştırmaları da fark üretmedi.

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Bu kanıt altı Direction quaternion sabitinin üretim sonucunu kapsar. Genel quaternion işlemleri, matrix dönüşümleri, Direction matrix rotation ve entity bakış yönü ayrı kapsamdır.
