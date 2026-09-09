# P01.b — Vec3 closerThan sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larını birlikte içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:120-122,145-150` içindeki iki gövde, ayrıca `util/Mth.java:634-644,702-704` içindeki `square` ve `lengthSquared` yardımcıları incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `closerThan(Position, double)`, üç koordinat farkının kareleri toplamını `distance * distance` ile karşılaştırır; karşılaştırma strict `<` kullanır.
- `closerThan(Vec3, double, double)`, x/z farklarının `dx * dx + dz * dz` toplamını `distanceXZ * distanceXZ` ile ve y farkının `Math.abs(dy)` değerini `distanceY` ile karşılaştırır; iki koşul da strict `<` olmalıdır.
- İkinci overload'da yatay koşul Java `&&` kısa devre sırasıyla önce değerlendirilir. Vec3 girdileri değişmez.
- Signed zero, taşma, Infinity ve NaN etkileri toleranssız boolean sonuçlarla karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | `position` | `xz_y` |
| --- | --- | ---: | ---: |
| `basic` | Sonlu üç boyutlu farklar ve iç eşikler | `true` | `true` |
| `strict-boundary` | 3-4-5 uzaklık ve eşit xz/y eşikleri | `false` | `false` |
| `vertical-gate` | Yalnız y farkı; tam y sınırı | `true` | `false` |
| `negative-zero-thresholds` | Signed zero farkları ve negatif sıfır eşikleri | `false` | `false` |
| `special-values` | NaN/Infinity koordinatları ve sonsuz eşikler | `false` | `false` |

Girdiler `family: "math"`, `operation: "vec3_closer_than"`, `first`, `second`, `distance`, `distance_xz` ve `distance_y` alanlarını taşır. Sonuç, `position` alanında Position overload'unu ve `xz_y` alanında yatay/y overload'unu boolean olarak içerir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::closer_than_position` ve `Vec3::closer_than_xz_y`, Java'nın fark, kare, toplama, strict karşılaştırma ve kısa devre sırasını taşır. `examples/behavior_adapter.rs` JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise iki overload çağrısını doğrudan Java sonuçlarına bağlar.
