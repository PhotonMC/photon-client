# P01.d — Shapes.INFINITY sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-infinity-parity.md)). Kapsam, `Shapes.INFINITY` sabitinin sınırsız tek shape olarak kurulmasını, box callback'ini ve aynı shape'in BooleanOp self sorgusunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:29-39`, `Shapes.java:44-52` ve `VoxelShape.java:117-139` incelendi. `MathOracle`, sabiti doğrudan kullanır; beklenen shape veya box listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `Shapes.INFINITY`, her eksende `Double.NEGATIVE_INFINITY` ile `Double.POSITIVE_INFINITY` arasını dolduran tek hücreli shape'tir.
- `forAllBoxes` bu shape için bir callback çağrısı yapar ve altı sonsuz endpoint'i Java callback sırasıyla taşır.
- Aynı infinity shape iki operand olarak `NOT_SAME` ile birleştirildiğinde bütün hücreler elenir ve sonuç empty shape olur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `snapshot` | Sabitin doğrudan shape snapshot'ı | Sonsuz box ve bounds |
| `boxes` | Sabitin `forAllBoxes` taraması | Callback uçları ve sırası |
| `not-same-self` | Aynı sabitin `NOT_SAME` birleşimi | Empty shape snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "infinity"` ve `view` alanlarını kullanır. `view`, snapshot, boxes veya not-same-self sorgusunu seçer.

## Java yakalama

Math referans kimliği `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80` oldu. Birleşik altı aile referansı `c8cad755994093a14efe4477c58b1b8fd21d851236fe6a0ef3d51d4be36aad80` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
