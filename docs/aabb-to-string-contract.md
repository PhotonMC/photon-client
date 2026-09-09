# P01.d — AABB toString sözleşmesi

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-to-string-parity.md)). Kapsam, Java `AABB.toString()` sonucunun endpoint metinlerini ve sabit ayraçlarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:433-436` incelendi. `MathOracle`, her sorguda Java `AABB.toString()` metodunu doğrudan çağırır; beklenen metni Rust çıktısından üretmez.

## Davranış sözleşmesi

- Sonuç `AABB[minX, minY, minZ] -> [maxX, maxY, maxZ]` biçimindedir.
- Altı endpoint, Java `Double.toString` kurallarına göre yazılır; `.0`, bilimsel gösterim, signed zero, `Infinity`, `-Infinity` ve `NaN` gözlenebilir.
- Constructor ile normalize edilen endpoint'ler yazdırılır; `toString()` kutunun durumunu değiştirmez.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Normal asimetrik endpoint'ler | AABB metni |
| `scientific` | Küçük/büyük exponent ve signed zero | Java double yazım biçimi |
| `signed-zero` | Sıfır endpoint'leri | İşaretli sıfır metni |
| `infinity` | Sonsuz endpoint'leri | Infinity metni |
| `nan` | NaN endpoint'i | NaN metni |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içinde `op: "to_string"` kullanır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math referans kimliği `3e58e449e5df13343950e698291673b0ac6cbfd83317a41d1c5806c3eab7f0f1`, birleşik altı aile referans kimliği `86041a1cb70cd52c496b0e0e6f4ee82e93ef833f7f480d3cc7f999fc562537e7` olarak doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
