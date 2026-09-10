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

Math referans kimliği `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`, birleşik altı aile referans kimliği `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583` olarak doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
