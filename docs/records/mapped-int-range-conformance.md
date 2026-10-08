# P05.a — OptionInstance IntRange.xmap sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, integer option range'inin map/from closure'larıyla başka bir tipe taşınan sliderable value set davranışını kapsar. Codec dönüşümü, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:282-353` ve `util/Mth.java:344-349,558-560` incelendi. Oracle gerçek `OptionInstance.IntRange.xmap` sonucunu `Integer`↔`String` map/from closure'larıyla çalıştırır.

- `validateValue`, giriş tipini `from` ile integer'a çevirir; base range doğrulaması başarılıysa sonucu `to` ile geri map eder.
- `toSliderValue` ve `fromSliderValue`, map/from dönüşümlerini base `IntRange` slider yollarına iletir.
- `discrete=true` iken `next`/`previous`, komşu integer değeri base range içinde ise döndürür; uçta mevcut mapped değeri korur. Geçerli olmayan mapped değerlerde base doğrulaması başarısız olduğundan mevcut integer index kullanılır.
- `discrete=false` iken `next` ve `previous` boş sonuç verir.

Rust karşılığı `crates/photon-game/src/options/values.rs` içindeki `MappedIntRange<R, To, From>` tipidir. Codec, `xmap` closure'larının hata üreten dönüşümleri ve GUI callback'leri bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "mapped_int_range"`, base min/max değerlerini, discrete bayrağını, mapped string değerlerini ve slider f64 bitlerini taşır. Sonuç validation, slider, next/previous ve `fromSliderValue` dönüşlerini taşır.


## Client-parity doğrulaması

`crates/photon-game/src/options/values.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.IntRangeBase.xmap` davranışını uygular. Java oracle gerçek `OptionInstance.IntRange.xmap` value set'ini ve mapped slider/validation yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `be5a11e648f87df6cf4cb1090e48fc53dd23c9708e4ee47076cd61375328ef6c`; tam corpus referansı `c12f2165094276efd6d8a12c03e1b2f5710d525f1116ecbf26748e0f95390abf` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-20uzgzrv` ve `compare-cmijp6if`; tam debug/release karşılaştırmaları `compare-lk20nrbw` ve `compare-og78u_zh` raporlarında, Java tekrar doğrulaması `verify-uqo8bj3v` raporunda fark üretmedi. Senaryolar discrete ve continuous map'leri, aralık içi/dışı mapped değerleri, uç komşuları ve özel slider değerlerini kapsar.

Codec xmap sonucu, closure hata davranışı, OptionInstance callback'leri ve GUI bu doğrulamanın kapsamı dışındadır.
