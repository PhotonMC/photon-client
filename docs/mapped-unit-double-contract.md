# P05.a — OptionInstance UnitDouble.xmap sözleşmesi

Bu dilim, `[0, 1]` double value set'inin from/to closure'larıyla başka bir tipe taşınan sliderable davranışını kapsar. Codec dönüşümü, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:580-623`, `client/Options.java:843-860` ve `util/Mth.java:554-560` incelendi. Oracle gerçek `OptionInstance.UnitDouble.xmap` sonucunu `Mth.square`/`Math.sqrt` closure'larıyla çalıştırır.

- `validateValue`, mapped değeri `from` ile double'a çevirir; `[0, 1]` doğrulaması başarılıysa sonucu `to` ile geri map eder.
- `toSliderValue`, `from` closure'ının sonucunu UnitDouble slider değerine taşır; değer clamp edilmez.
- `fromSliderValue`, UnitDouble slider değerini `to` closure'ı ile mapped tipe dönüştürür.
- Java f64 signed zero, subnormal ve özel değer işlemleri closure'ların aritmetik/sqrt yollarında korunur.

Rust karşılığı `src/option.rs` içindeki `MappedUnitDouble<R, To, FromValue>` tipidir. Codec, closure hata davranışı ve GUI callback'leri bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "mapped_unit_double"`, mapped f64 değerlerini ve slider f64 bitlerini taşır. Sonuç validation, mapped slider ve `fromSliderValue` f64 bitlerini taşır.
