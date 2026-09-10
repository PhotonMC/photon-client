# P05.a — OptionInstance UnitDouble.xmap doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.UnitDouble.xmap` davranışını uygular. Java oracle gerçek `UnitDouble.xmap(Mth::square, Math::sqrt)` value set'ini ve mapped slider/validation yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `a1327aaf9a6b54c6de32367b961063c8b94d51b45eb95ee82d8a111d64fa4828`; tam corpus referansı `3383066e0cdaae0781a501c0635e4e925ff35daa2cce5878173cbe0f799aeb5b` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-v1aldqpi` ve `compare-fb6cslb4`; tam debug/release karşılaştırmaları `compare-r5rzrfi5` ve `compare-o9yewgj0` raporlarında, Java tekrar doğrulaması `verify-mwhezoes` raporunda fark üretmedi. Senaryolar mapped validation, slider yönlendirmesi, signed zero, subnormal, sınır ve özel f64 değerlerini kapsar.

Codec xmap sonucu, closure hata davranışı, OptionInstance callback'leri ve GUI bu doğrulamanın kapsamı dışındadır.
