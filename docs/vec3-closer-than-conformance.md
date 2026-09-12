# P01.b — Vec3 closerThan sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larının strict eşiklerini, x/z toplama sırasını ve y koşulunu karşılar. Java oracle iki overload'u doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda iç eşikler, strict sınır eşitliği, ayrı y kapısı, negatif sıfır eşikleri ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki closerThan overload'unun boolean sonuçlarıyla sınırlıdır. `distanceToSqr` ve Mth yardımcıları ayrı dilimlerdir; Position arayüzünün Vec3 dışındaki uygulamaları bu adaptörde temsil edilmez.
