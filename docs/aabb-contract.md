# P01.d — AABB contract sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-parity.md)). Kapsam, hareket delta'sının işaretine göre Java `AABB.contract(double,double,double)` çağrısının kutunun tek uç noktasını içeri taşımasını içerir. AABB inflate için ayrı [sözleşme](aabb-inflate-contract.md), intersect için ayrı [sözleşme](aabb-intersect-contract.md), move için ayrı [sözleşme](aabb-move-contract.md), sizes için ayrı [sözleşme](aabb-sizes-contract.md) ve hasNaN için ayrı [sözleşme](aabb-has-nan-contract.md), positions için ayrı [sözleşme](aabb-positions-contract.md), axis min/max için ayrı [sözleşme](aabb-axis-limits-contract.md), distanceToSqr için ayrı [sözleşme](aabb-distance-contract.md), ofSize için ayrı [sözleşme](aabb-of-size-contract.md), endpoint setter'ları için ayrı [sözleşme](aabb-set-endpoint-contract.md) vardır; clip ve diğer genişletme/kesişim işlemleri ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:129-155` incelendi. `MathOracle` doğrudan `AABB.contract` çağırır; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Negatif delta alt endpoint'i `min -= delta` ile içeri taşır; pozitif delta üst endpoint'i `max -= delta` ile içeri taşır.
- Sıfır ve signed zero delta'ları hiçbir endpoint'i değiştirmez. Her eksen bağımsız işlenir.
- Sonuç Java `AABB` kurucusundan geçer; endpoint'lerin eksen bazlı min/max normalizasyonu korunur.
- Bu dilimde zincir adımları bir önceki `AABB` sonucunu sonraki çağrıya verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Tek yön | 2 | Tüm eksenlerde pozitif veya negatif contract delta'sı |
| Karışık işaret | 1 | Aynı çağrıda negatif, sıfır ve pozitif delta |
| Sıfır/zincir | 2 | Signed zero ve ardışık contract çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "contract"` ve f64 bit string `x`, `y`, `z` delta'ları bulunur. Her adım sonrası kutu uçları f64 bit string olarak `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Contract senaryolarını içeren math referans kimliği `3854d607b2b0b2e782a153bffb5f46917835f2df40d4a855940d960953a37113` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 371/371, birleşik corpus'ta 1449/1449 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
