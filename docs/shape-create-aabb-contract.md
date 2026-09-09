# P01.d — Shapes.create(AABB) sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-create-aabb-parity.md)). Kapsam, `Shapes.create(AABB)` overload'ının AABB endpoint'lerini scalar create yoluna aktarmasını, AABB normalizasyonunu ve küçük aralık eşiğini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:52-99` ve `world/phys/AABB.java:22-29` incelendi. `MathOracle`, `Shapes.create(AABB)` çağrısını doğrudan yapar; beklenen shape listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `Shapes.create(AABB)` AABB'nin altı normalize edilmiş endpoint'ini `create(double...)` yoluna taşır.
- AABB constructor'ı her eksende `Math.min`/`Math.max` ile normalizasyon yaptığı için ters verilen giriş endpoint'leri create çağrısına sıralı ulaşır.
- Herhangi bir eksen genişliği `1e-7` değerinden küçükse sonuç empty shape olur. Diğer girdiler scalar create yolunun coordinate grid ve box üretim davranışını korur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `reversed` | Her eksende ters endpoint'li AABB | AABB normalizasyonundan sonra oluşan shape snapshot'ı |
| `empty` | Bir ekseni `1e-7` altı olan AABB | Empty shape snapshot'ı |
| `array` | Birim küp dışına taşan ve grid'e oturmayan endpoint'ler | Array shape'in box ve bounds snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "create_aabb"` ve `aabb` alanlarını kullanır. Sonuç scalar shape snapshot formatındaki `empty`, `boxes` ve non-empty durumda `bounds` alanlarını taşır.

## Java yakalama

Math referans kimliği `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec` oldu. Birleşik altı aile referansı `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
