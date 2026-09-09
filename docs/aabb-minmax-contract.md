# P01.d — AABB minmax sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-minmax-parity.md)). Kapsam, Java `AABB.minmax(AABB)` çağrısının iki kutuyu her eksende kapsayan yeni AABB'yi üretmesini ve sonucu AABB kurucusundan geçirmesini içerir. AABB contract, inflate, intersect, move, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize ve clip ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:213-221` incelendi. `MathOracle` doğrudan `AABB.minmax` çağrısını yapar; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Her eksenin min endpoint'i iki kutunun min değerlerinin `Math.min` sonucudur.
- Her eksenin max endpoint'i iki kutunun max değerlerinin `Math.max` sonucudur.
- Sonuç Java `AABB` kurucusundan geçer; ayrık kutuların union'ı aradaki boşluğu da kapsayan tek kutuya dönüşür.
- Yüz yüze temas eden kutuların union'ı temas eksenindeki dış uçları korur. Signed zero seçimleri Java `Math.min`/`Math.max` ve kurucu semantiğine bağlıdır.
- Zincir adımları bir önceki union sonucunu sonraki `minmax` çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Örtüşme | 2 | İç içe ve kısmi eksen örtüşmesi |
| Sınır | 2 | Ayrık kutu ve yüz teması |
| Sayısal ayrıntı | 1 | Signed zero endpoint'leri |
| Zincir | 1 | Ardışık union çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "minmax"` ve `box` nesnesi bulunur. Kutu uçları f64 bit string olarak verilir; her adım sonrası sonuç `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Minmax senaryolarını içeren math referans kimliği `a5af108bd9af6e253a66e4f7d2ec37609cab22b050af5da64c032db59e8c71c2` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `a15082ba584c6d0bca4bc52d80fd5da8fc49f05193cec75afd71cf0af0c4ee0d` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 388/388, birleşik corpus'ta 1594/1594 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
