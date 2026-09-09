# P01.d — Shapes.EPSILON ve BIG_EPSILON sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-epsilon-parity.md)). Kapsam, Java `Shapes` sınıfının public `EPSILON` ve `BIG_EPSILON` sabit değerlerini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:20-22` incelendi. `MathOracle`, sabitleri doğrudan okur; değerler Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Shapes.EPSILON` tam olarak `1.0E-7` double değeridir.
- `Shapes.BIG_EPSILON` tam olarak `1.0E-6` double değeridir.
- JSONL sonucu değerleri tolerans uygulamadan ham f64 bitleriyle karşılaştırır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `epsilon` | `Shapes.EPSILON` sorgusu | Ham f64 bitleri |
| `big-epsilon` | `Shapes.BIG_EPSILON` sorgusu | Ham f64 bitleri |
| `both` | İki sabitin birlikte sorgusu | Her iki ham f64 değeri |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "constants"` ve `name` alanlarını kullanır.

## Java yakalama

Math referans kimliği `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126` oldu. Birleşik altı aile referansı `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
