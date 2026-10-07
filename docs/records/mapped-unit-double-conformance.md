# P05.a — OptionInstance UnitDouble.xmap sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, `[0, 1]` double value set'inin from/to closure'larıyla başka bir tipe taşınan sliderable davranışını kapsar. Codec dönüşümü, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:580-623`, `client/Options.java:843-860` ve `util/Mth.java:554-560` incelendi. Oracle gerçek `OptionInstance.UnitDouble.xmap` sonucunu `Mth.square`/`Math.sqrt` closure'larıyla çalıştırır.

- `validateValue`, mapped değeri `from` ile double'a çevirir; `[0, 1]` doğrulaması başarılıysa sonucu `to` ile geri map eder.
- `toSliderValue`, `from` closure'ının sonucunu UnitDouble slider değerine taşır; değer clamp edilmez.
- `fromSliderValue`, UnitDouble slider değerini `to` closure'ı ile mapped tipe dönüştürür.
- Java f64 signed zero, subnormal ve özel değer işlemleri closure'ların aritmetik/sqrt yollarında korunur.

Rust karşılığı `crates/voxel-game/src/option.rs` içindeki `MappedUnitDouble<R, To, FromValue>` tipidir. Codec, closure hata davranışı ve GUI callback'leri bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "mapped_unit_double"`, mapped f64 değerlerini ve slider f64 bitlerini taşır. Sonuç validation, mapped slider ve `fromSliderValue` f64 bitlerini taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.UnitDouble.xmap` davranışını uygular. Java oracle gerçek `UnitDouble.xmap(Mth::square, Math::sqrt)` value set'ini ve mapped slider/validation yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `a1327aaf9a6b54c6de32367b961063c8b94d51b45eb95ee82d8a111d64fa4828`; tam corpus referansı `3383066e0cdaae0781a501c0635e4e925ff35daa2cce5878173cbe0f799aeb5b` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-v1aldqpi` ve `compare-fb6cslb4`; tam debug/release karşılaştırmaları `compare-r5rzrfi5` ve `compare-o9yewgj0` raporlarında, Java tekrar doğrulaması `verify-mwhezoes` raporunda fark üretmedi. Senaryolar mapped validation, slider yönlendirmesi, signed zero, subnormal, sınır ve özel f64 değerlerini kapsar.

Codec xmap sonucu, closure hata davranışı, OptionInstance callback'leri ve GUI bu doğrulamanın kapsamı dışındadır.
