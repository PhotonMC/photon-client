# P01.d — AABB intersect sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-intersect-parity.md)). Kapsam, Java `AABB.intersect(AABB)` çağrısının iki kutunun her eksendeki ortak aralığını üretmesini ve sonucu AABB kurucusundan geçirmesini içerir. Predicate overload'ları için ayrı [sözleşme](aabb-predicate-overloads-contract.md) vardır; AABB contract, inflate, minmax, move, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize ve clip ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:203-211` incelendi. `MathOracle` doğrudan `AABB.intersect` çağrısını yapar; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Her eksenin min endpoint'i iki kutunun min değerlerinin `Math.max` sonucudur.
- Her eksenin max endpoint'i iki kutunun max değerlerinin `Math.min` sonucudur.
- Ham ortak aralık ters dönse bile sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer; ayrık kutular bu nedenle aradaki aralığı temsil eden normalize edilmiş degenerate olmayan kutu üretebilir.
- Yüz yüze temas eden kutular ilgili eksende min ve max'ın eşit olduğu degenerate bir sonuç üretir. Signed zero seçimleri Java `Math.min`/`Math.max` ve kurucu semantiğine bağlıdır.
- Zincir adımları bir önceki kesişim sonucunu sonraki `intersect` çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Örtüşme | 2 | İç içe ve kısmi eksen örtüşmesi |
| Sınır | 2 | Ayrık kutunun normalizasyonu ve yüz teması |
| Sayısal ayrıntı | 1 | Signed zero endpoint'leri |
| Zincir | 1 | Ardışık kesişim çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "intersect"` ve `box` nesnesi bulunur. Kutu uçları f64 bit string olarak verilir; her adım sonrası sonuç `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Intersect senaryolarını içeren math referans kimliği `067168f58b7d3c483bc26a542577bb0191d5cf0956a168754322b8805241bae5` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `b0cbcb74aa0f40450d03ba7ed23e6155f13776414b6dc51e556a0b02e08fa2c4` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 382/382, birleşik corpus'ta 1037/1037 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
