# P01.d — AABB inflate sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-inflate-parity.md)). Kapsam, Java `AABB.inflate` üç eksenli ve tek miktarlı overload'larının her iki endpoint'i delta kadar dışarı taşımasını içerir. AABB contract, deflate üç eksenli overload, intersect, minmax, move, sizes, hasNaN, positions, axis min/max, distanceToSqr ve ofSize için ayrı sözleşmeler, clip ve diğer kesişim işlemleri ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:189-201` incelendi. `MathOracle` doğrudan `AABB.inflate` overload'larını çağırır; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Üç eksenli overload'da her eksenin min endpoint'inden delta çıkarılır, max endpoint'ine delta eklenir.
- Scalar overload aynı miktarı üç eksene verir. Negatif miktar kutuyu daraltır; sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer.
- Sıfır ve signed zero miktarları endpoint'leri değiştirmez. Eksenler bağımsız işlenir.
- Zincir adımları bir önceki `AABB` sonucunu sonraki overload çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Scalar overload | 2 | Pozitif ve negatif tek miktar |
| Üç eksenli overload | 1 | Karışık işaretli x/y/z delta'ları |
| Sıfır/zincir | 2 | Signed zero ve ardışık overload çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "inflate"` scalar `amount` veya `op: "inflate_xyz"` f64 bit string `x`, `y`, `z` delta'ları bulunur. Her adım sonrası kutu uçları f64 bit string olarak `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Inflate senaryolarını içeren math referans kimliği `cc1e4e363fee8a21012ea0d56765f85c05556ced56a4bd8c330f603a71ce5656` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `7c9538198afb189201c05520d1d9beb9d8b8ed1ecf818a5751ba7c9cf2774c30` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 376/376, birleşik corpus'ta 1375/1375 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
