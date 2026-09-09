# P01.d — Shapes.or varargs referans sözleşmesi

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-or-many-parity.md)). Kapsam, Java `Shapes.or(VoxelShape first, VoxelShape... tail)` overload'ının ilk shape ve varargs kuyruğunu birleştirme davranışıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:117-120`, `BooleanOp.java:3-21` ve `VoxelShape.java:29-121` incelendi. `MathOracle`, varargs overload'ını doğrudan çağırır; beklenen shape kutuları test içinde yeniden hesaplanmaz.

## Davranış sözleşmesi

- İlk shape zorunludur; tail boşsa Java overload'ı ilk shape'i sonuç olarak korur.
- Her tail öğesi, önceki sonuçla soldan sağa `Shapes.or` ve `BooleanOp.OR` yolundan birleştirilir.
- Birleşim sonucu Java `toAabbs` sırasındaki kutular, `empty` durumu ve non-empty `bounds` alanlarıyla gözlenir.
- Empty ilk shape, sonraki non-empty shape'lerin birleşimini engellemez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| touching-row | 1 | Yüzeyden temas eden ve birlikte tam blok oluşturan üç box |
| disjoint | 1 | Ayrık konumlarda üç box ve farklı koordinat düzlemleri |
| empty-first | 1 | Empty olan ilk box ve art arda birleşen iki box |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "join_many"` ve `shapes` dizisini kullanır. Sonuç `shape` alanında shape snapshot'ı taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimdeki senaryoların bulunduğu güncel math referansı `ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `ac2c4ad5c101cb49e75f0ac8ce84fd9925829064f84fb224dd8fcb6107a507bc` kimliğiyle doğrulandı.

Math debug/release raporları `compare-vn_8nl_6` ve `compare-e350pb7p`; birleşik debug/release raporları `compare-bdb0mgf_` ve `compare-0a9n4nfd` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
