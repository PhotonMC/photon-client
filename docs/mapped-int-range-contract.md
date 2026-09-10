# P05.a — OptionInstance IntRange.xmap sözleşmesi

Bu dilim, integer option range'inin map/from closure'larıyla başka bir tipe taşınan sliderable value set davranışını kapsar. Codec dönüşümü, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:282-353` ve `util/Mth.java:344-349,558-560` incelendi. Oracle gerçek `OptionInstance.IntRange.xmap` sonucunu `Integer`↔`String` map/from closure'larıyla çalıştırır.

- `validateValue`, giriş tipini `from` ile integer'a çevirir; base range doğrulaması başarılıysa sonucu `to` ile geri map eder.
- `toSliderValue` ve `fromSliderValue`, map/from dönüşümlerini base `IntRange` slider yollarına iletir.
- `discrete=true` iken `next`/`previous`, komşu integer değeri base range içinde ise döndürür; uçta mevcut mapped değeri korur. Geçerli olmayan mapped değerlerde base doğrulaması başarısız olduğundan mevcut integer index kullanılır.
- `discrete=false` iken `next` ve `previous` boş sonuç verir.

Rust karşılığı `src/option.rs` içindeki `MappedIntRange<R, To, From>` tipidir. Codec, `xmap` closure'larının hata üreten dönüşümleri ve GUI callback'leri bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "mapped_int_range"`, base min/max değerlerini, discrete bayrağını, mapped string değerlerini ve slider f64 bitlerini taşır. Sonuç validation, slider, next/previous ve `fromSliderValue` dönüşlerini taşır.
