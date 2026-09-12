# P01.c — BlockPos between corners in direction sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, yön işaretine göre başlayan ve eksen önceliğine göre ilerleyen BlockPos köşe traversal'ını ölçer.

## Kaynak ve sınır

`.data/minecraft/source/net/minecraft/core/BlockPos.java:524-616` ve `Direction.java:379-380` incelendi. AABB overload'u normalize edilmiş uçları floor eder; BlockPos ve ham koordinat overload'ları ortak int yoluna yönlenir. Ortak yol, uçları eksen bazında normalize eder, yön bileşenleriyle başlangıç köşesini seçer, X/Z mutlak büyüklüklerinin strict karşılaştırmasıyla Y-X-Z veya Y-Z-X eksen sırasını kurar ve son ekseni en hızlı değiştirir.

## Davranış sözleşmesi

- AABB min/max uçları önce `Mth.floor` ile integer köşelere çevrilir.
- Her overload aynı normalize edilmiş integer koordinat traversal'ını kullanır.
- Direction bileşeni `>= 0.0` ise ilgili eksende minimum köşe ve pozitif adım, aksi halde maksimum köşe ve negatif adım seçilir; signed zero pozitif dalı izler, NaN negatif dalı izler.
- `abs(x) < abs(z)` yalnızca true olduğunda Y-Z-X, diğer tüm sonuçlarda Y-X-Z eksen sırası seçilir.
- Üçüncü eksen en hızlı, ikinci eksen onun bitiminde, birinci eksen en yavaş ilerler; her nokta bağımsız BlockPos snapshot'ı olarak gözlenir.
- Farklar Java i32 çıkarma semantiğiyle hesaplanır.

## JSONL arayüzü

İstek `family: "math"`, `operation: "blockpos_between_corners_direction"`, bir AABB, iki BlockPos, iki ham integer köşe ve f64 bit alanlı direction taşır. Sonuç `aabb`, `block_pos` ve `coordinates` dizilerini içerir.

Java oracle üç overload'u ayrı ayrı tüketir. Rust karşılığı `BlockPos::between_corners_in_direction_aabb`, `between_corners_in_direction` ve `between_corners_in_direction_coordinates` yordamlarıdır.

## Kapsam dışı

`MutableBlockPos` mutasyon API'si, breadth-first traversal, dünya collider toplama ve AABB dışındaki iterator kaynakları bu dilime dahil değildir.


## Client-parity doğrulaması

Rust adapter `blockpos_between_corners_direction` işlemi, AABB, BlockPos ve ham koordinat overload'larını Java oracle ile karşılaştırır.

Java math capture kimliği `12fd4fac2ab5b4c789f5326d449ac86f9727e7ab6c66cb23a0cda6291d545d03` olarak `.data/captures/behavior/` altında tutulur. İki JVM çalıştırmasında kararlı bulundu; Rust debug adapter karşılaştırması aynı kimliğe karşı başarılı oldu.

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py verify --family math
```

Bu kanıt AABB floor forwarding'ini, normalize edilmiş köşeleri, Y-X-Z/Y-Z-X eksen önceliğini, yön işaretlerini, signed zero/NaN dallarını ve i32 fark taşmasını kapsar.

Tüm davranış ailelerini kapsayan son ortak capture kimliği `a484457fbd28df4964559a6bd7ceb4bf1a54df8b623de1cb84048c96b7df3f38`'dir. Debug ve release adapter'ları bu capture'a karşı başarılı oldu.
