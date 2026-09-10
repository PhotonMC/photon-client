# P05.a — OptionInstance LazyEnum doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.LazyEnum` davranışını uygular. Java oracle gerçek `LazyEnum` supplier, validation function ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `98e05ab8593dd463cfcb291851a8d40a7811a0177c62b55ee18df108bb63a001`; tam corpus referansı `b61a7e72e2f39b12b39f624fd688f784b59e627eda5084fe25156cca0fbfb83c` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-nrf3ujvg` ve `compare-xp6rhro8`; tam debug/release karşılaştırmaları `compare-5ltllvat` ve `compare-7fl9g6ro` raporlarında, Java tekrar doğrulaması `verify-r_rotijy` raporunda fark üretmedi. Senaryolar supplier'ın çağrı başına liste değiştirmesini, immutable snapshot sırasını ve liste dışındaki bir değeri kabul edebilen bağımsız validation function'ı kapsar.

CycleButton'ın gerçek GUI olayları, option callback'leri, Codec ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
