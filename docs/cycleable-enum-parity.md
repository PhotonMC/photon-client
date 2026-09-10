# P05.a — OptionInstance Enum doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.Enum` value list ve validation davranışını uygular. Java oracle gerçek Java `OptionInstance.Enum<String>` ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `5aea1312d36c1500e8c18430f34e9c7d2debdb0ab3bf52d6adf7ec327b31aeef`; tam corpus referansı `46b1b9991e2bbfe2deb0ab630626cd047fa111697cf78408552a34159fed0266` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-k5s4yghv` ve `compare-pge6l_6l`; tam debug/release karşılaştırmaları `compare-c8vviwok` ve `compare-yvvp73ji` raporlarında, Java tekrar doğrulaması `verify-cg051ddh` raporunda fark üretmedi.

CycleButton'ın gerçek GUI olayları, tıklama/scroll modulo döngüsü, option callback'leri, Codec ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
