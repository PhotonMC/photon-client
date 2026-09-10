# P05.a — OptionInstance ClampingLazyMaxIntRange doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.ClampingLazyMaxIntRange` davranışını uygular. Java oracle gerçek dinamik max supplier'ı, inherited `IntRangeBase` yollarını ve `CycleButton.ValueListSupplier` listesini çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `a609ada324e896ae42aa8fc7f01c20b288289aede5d9e0b1687c3fa5cd4c4f60`; tam corpus referansı `18d1766f46ca8c601275e6c300f05426f971447bf83491eeba96fba4be6b568e` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-3e2fz9jh` ve `compare-ugv8j904`; tam debug/release karşılaştırmaları `compare-v_srm0lz` ve `compare-3iazkjm7`; Java tekrar doğrulaması `verify-kwmjb6el` raporlarında fark üretmedi. Senaryolar dinamik üst sınır erişimini, clamp edilen değerleri, i32 next/previous, slider sınırlarını, özel slider değerlerini ve cycle button listesini kapsar.

Codec, gerçek `Window.calculateScale`, VideoSettingsScreen olayları, option callback'leri ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
