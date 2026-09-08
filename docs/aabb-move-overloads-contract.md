# P01.d — AABB move overload sözleşmesi

Bu dilim, Java `AABB.move(BlockPos)`, `AABB.move(Vec3)` ve `AABB.move(Vector3f)` overload'larının kutuyu taşımasını kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](aabb-move-overloads-parity.md).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:223-239` içindeki üç overload ve scalar `move` çağrısı incelendi. `BlockPos` bileşenlerinin i32, `Vec3` bileşenlerinin f64, `Vector3f` bileşenlerinin f32 olduğu sabitlendi. `MathOracle` her overload'u doğrudan çağırır; beklenen uçları oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `BlockPos` overload'u üç tamsayı koordinatı Java sayısal genişletmesiyle double toplama ifadelerine taşır.
- `Vec3` overload'u üç double bileşenini scalar `move` yoluna iletir.
- `Vector3f` overload'u üç float bileşenini double toplama öncesinde Java'nın widening dönüşümüyle taşır; f32 bitleri bu nedenle girdinin parçasıdır.
- Her endpoint aynı eksendeki miktarla toplanır ve sonuç yeni `AABB` kurucusundan geçer.
- Signed zero, sonsuzluk, NaN ve büyük değerlerde oluşan IEEE-754 double bitleri gözlenebilir çıktının parçasıdır.
- Overload çağrıları zincirlenebilir; her sonuç sonraki çağrının kutusudur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Temel zincir | 1 | BlockPos, Vec3 ve Vector3f overload'larını sırayla çağırır |
| BlockPos sınırları | 1 | i32 minimum/maksimum koordinatları |
| Vec3 özel değerleri | 1 | Sonsuzluk, NaN ve normal double bileşenleri |
| Vector3f hassasiyeti | 1 | f32 olarak kodlanan kesirli ve büyük bileşenler |
| Signed zero | 1 | Üç overload'da signed zero ve sıfır BlockPos |
| Karışık zincir | 1 | Overload'ları farklı sırada ardışık çağırır |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "move_block_pos"`, `move_vec3` veya `move_vector3f` bulunur. `position` nesnesi BlockPos için decimal i32 dizgeleri, Vec3 için f64 ve Vector3f için f32 bit dizgeleri taşır. Her adım kutu snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Overload senaryolarını içeren math referans kimliği `e7ae22f078eb9523961cde345ba3ea72bd43b52904e0c750f321f6b4663fd1cd` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](aabb-move-overloads-parity.md).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
