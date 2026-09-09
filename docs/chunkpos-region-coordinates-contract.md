# P01.c — ChunkPos region koordinatı yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `ChunkPos` değerinden region koordinatlarını ve 32×32 region içindeki yerel indeksleri çıkarmak; packed ChunkPos değerinden aynı region koordinatlarını okumak; ayrıca region koordinatından ilk ve son chunk'ı üretmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:53-59` içindeki `minFromRegion`/`maxFromRegion` ile `world/level/ChunkPos.java:128-150` içindeki instance/static region ve local yardımcıları incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `ChunkPos` packing biçiminin tamamı, `isValid`, `hash`, block koordinatı yardımcıları, `BlockPos` dönüşümleri ve range/distance yardımcıları ölçülmez. Packed region sorguları için packed değer aynı `ChunkPos` nesnesinin Java `pack()` sonucu olarak oluşturulur.

## Davranış sözleşmesi

- `getRegionX/Z`, chunk koordinatını 5 bit aritmetik sağa kaydırır; negatif değerlerde sonuç sıfıra doğru kesilmez.
- Static `getRegionX/Z(long)`, packed long içinden signed x/z alanlarını çıkarır ve aynı 5 bit aritmetik kaydırmayı uygular.
- `getRegionLocalX/Z`, chunk koordinatının düşük 5 bitini maskeler ve 0–31 aralığındaki region içi indeksi verir.
- `minFromRegion`, region koordinatını 5 bit sola kaydırarak ilk chunk'ı üretir.
- `maxFromRegion`, aynı ilk chunk koordinatlarına 31 ekleyerek son chunk'ı üretir.
- Java int kaydırma ve toplama taşmaları i32 içinde sarar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Region koordinatı yardımcıları | 6 | Köken, pozitif/negatif region sınırları, karışık koordinatlar, shift taşması ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "chunkpos_region_coordinates"`, chunk `x`/`z` ve region `region_x`/`region_z` alanlarını kullanır. `value` altında instance region, packed region, local indeks ve region'dan üretilen min/max `ChunkPos` snapshot'ları taşınır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::get_region_x`, `get_region_z`, `get_region_x_from_packed`, `get_region_z_from_packed`, `get_region_local_x`, `get_region_local_z`, `min_from_region` ve `max_from_region` Java'nın shift, mask ve i32 wrapping aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.
