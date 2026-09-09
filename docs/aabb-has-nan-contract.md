# P01.d — AABB hasNaN sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-has-nan-parity.md)). Kapsam, Java `AABB.hasNaN()` sorgusunun kutuda saklanan altı endpoint'in herhangi birinde NaN bulunup bulunmadığını bildirmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:438-445` incelendi. `MathOracle` `hasNaN()` metodunu doğrudan çağırır; endpoint'leri yeniden tarayan bir oracle hesabı kullanmaz.

## Davranış sözleşmesi

- `minX`, `minY`, `minZ`, `maxX`, `maxY` ve `maxZ` sırayla `Double.isNaN` ile kontrol edilir.
- Herhangi bir saklanan endpoint NaN ise sonuç `true`, altı endpoint de sonlu veya sonsuz fakat NaN değilse sonuç `false` olur.
- AABB kurucusunun `Math.min`/`Math.max` normalizasyonu korunur; bir eksenin giriş uçlarından biri NaN olduğunda o eksenin saklanan uçları Java davranışına göre NaN olabilir.
- Signed zero ve pozitif/negatif sonsuz NaN sayılmaz. Sorgu kutuyu değiştirmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Sonlu | 1 | NaN içermeyen birim kutu |
| NaN yayılımı | 3 | X alt ucu, Y üst ucu ve Z'nin iki ucu |
| Sonsuzluk | 2 | Pozitif ve negatif sonsuz uçlar |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "has_nan"` bulunur. Her sonuç `has_nan` boolean alanını taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

HasNaN senaryolarını içeren math referans kimliği `1b628a5ed1aa6d5d7f23933491eed86b44ffdbc00f80560b0ddbe51afb57e990` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `cf0e137848a347df8fce62ca16437dc60c6c7662652e688c9790d83f2d52de0b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 406/406, birleşik corpus'ta 957/957 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
