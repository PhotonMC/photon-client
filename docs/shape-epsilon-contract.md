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

Math referans kimliği `72703f27997a3fbdb61c87ead93863a7a53806ebb3533f7e4c676f9a81591df9` oldu. Birleşik altı aile referansı `e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
