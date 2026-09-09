# P01.d — AABB axis min/max sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-axis-limits-parity.md)). Kapsam, Java `AABB.min(Direction.Axis)` ve `AABB.max(Direction.Axis)` sorgularının seçilen eksenin saklanan endpoint'ini döndürmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:87-94` ve eksen seçiminin `core/Direction.java:402-544` içindeki `Axis.choose` davranışı incelendi. `MathOracle` getter'ları doğrudan çağırır; endpoint değerleri oracle içinde yeniden seçilmez.

## Davranış sözleşmesi

- X ekseni `minX`/`maxX`, Y ekseni `minY`/`maxY`, Z ekseni `minZ`/`maxZ` çiftini seçer.
- Sorgu aritmetik yapmaz; seçilen endpoint'in f64 bitleri NaN ve signed zero dahil aynen döner.
- Constructor normalizasyonu getter'dan önce uygulanır. Sorgu kutuyu değiştirmez.
- `minmax(AABB)` kapsamı ayrı dilimde doğrulanır; burada yalnızca bir `Direction.Axis` seçimi vardır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Eksen seçimi | 3 | X/Y/Z için tüm min endpoint'leri |
| Üst endpoint | 1 | X/Y/Z için tüm max endpoint'leri |
| Karışık sorgu | 1 | Farklı sırada min ve max çağrıları |
| Sayısal ayrıntı | 1 | Ters sıralı, signed zero ve NaN endpoint'leri |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "min"` veya `op: "max"` ve `axis: "x" | "y" | "z"` bulunur. Her sonuç seçilen endpoint'i `value` f64 bit string alanında taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Axis min/max senaryolarını içeren math referans kimliği `803f0124e49c7979da9fe752dd944ea1e0edce936d4d6afa25b3ab5358a5514c` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 418/418, birleşik corpus'ta 1569/1569 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
