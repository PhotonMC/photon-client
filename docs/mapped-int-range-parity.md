# P05.a — OptionInstance IntRange.xmap doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.IntRangeBase.xmap` davranışını uygular. Java oracle gerçek `OptionInstance.IntRange.xmap` value set'ini ve mapped slider/validation yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `be5a11e648f87df6cf4cb1090e48fc53dd23c9708e4ee47076cd61375328ef6c`; tam corpus referansı `c12f2165094276efd6d8a12c03e1b2f5710d525f1116ecbf26748e0f95390abf` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-20uzgzrv` ve `compare-cmijp6if`; tam debug/release karşılaştırmaları `compare-lk20nrbw` ve `compare-og78u_zh` raporlarında, Java tekrar doğrulaması `verify-uqo8bj3v` raporunda fark üretmedi. Senaryolar discrete ve continuous map'leri, aralık içi/dışı mapped değerleri, uç komşuları ve özel slider değerlerini kapsar.

Codec xmap sonucu, closure hata davranışı, OptionInstance callback'leri ve GUI bu doğrulamanın kapsamı dışındadır.
