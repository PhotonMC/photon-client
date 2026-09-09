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

Math referans kimliği `ba2c0a50dc0b1df009707b281c6734a888c59ecbfee1929dc86125f395920582`, birleşik altı aile referans kimliği `3be9baa80520a67f762a0d1221d9e4a40892597f1dc8ccba80afd18b3fe8689b` olarak doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
