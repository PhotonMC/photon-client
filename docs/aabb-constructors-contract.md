# P01.d — AABB kutu kurucuları ve üreticileri sözleşmesi

Bu dilim, Java `AABB(BlockPos)`, `AABB(Vec3, Vec3)`, `AABB.unitCubeFromLowerCorner(Vec3)` ve `AABB.encapsulatingFullBlocks(BlockPos, BlockPos)` yollarını kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](aabb-constructors-parity.md).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:22-55` içindeki dört kutu kurucusu/üreticisi incelendi. `MathOracle` her adımda Java çağrısını doğrudan yapar; beklenen endpoint'leri oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `AABB(BlockPos)` BlockPos origin'i ile origin + 1 arasındaki unit box'ı üretir. Origin + 1 Java int işlemidir; i32 sınırında taşma gerçekleşmeden double kurucuya geçilmez.
- `AABB(Vec3, Vec3)` iki Vec3 bileşenini altı endpoint'li kurucuya aktarır. Kurucu her eksende `Math.min`/`Math.max` ile endpoint'leri normalize eder; NaN ve signed zero bu semantiği izler.
- `unitCubeFromLowerCorner(Vec3)` alt köşe bileşenlerini doğrudan min endpoint'lerine, her bileşene double `1.0` eklenmiş değerleri max endpoint'lerine verir.
- `encapsulatingFullBlocks(BlockPos, BlockPos)` her eksende iki integer origin'in min değerini alt, max değerine Java int `+1` uygulanmış değeri üst endpoint olarak seçer.
- Kurucular yeni AABB sonucu döndürür; sorgu zincirinde önceki kutunun yerini bu sonuç alır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| BlockPos kurucusu | 1 | Negatif koordinatlar ve i32 uç değerleri |
| Vec3 kurucusu | 1 | Ters endpoint'ler, signed zero ve özel değerler |
| Unit cube | 1 | Normal, signed zero, sonsuzluk ve NaN alt köşeleri |
| Full block kapsamı | 1 | Ters sıradaki iki BlockPos ve negatif koordinatlar |
| Full block taşması | 1 | Integer minimum/maksimumda üst endpoint `+1` davranışı |
| Kurucu zinciri | 1 | Dört üretici yolunun sıralı çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "from_block_pos"`, `from_vec3`, `unit_cube_from_lower_corner` veya `encapsulating_full_blocks` bulunur. BlockPos koordinatları decimal i32 dizgeleri, Vec3 koordinatları f64 bit dizgeleri taşır. Her adım kutu endpoint'lerinin f64 bit snapshot'ını ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimin math referans kimliği `5dc02c58d7cf60cb11342c786c6a4d91d4d727099b7ac1faef9f51a5b5f192df` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](aabb-constructors-parity.md).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
