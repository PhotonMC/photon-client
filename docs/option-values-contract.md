# P05.a — OptionInstance value set sözleşmesi

Bu dilim, ayar widget'ı veya dosya yükleme gerektirmeyen `IntRange` ve `UnitDouble` value set davranışını kapsar. OptionInstance'ın genel value taşıması, Codec JSON serileştirmesi, seçenek dosyası ve GUI callback akışı sonraki ayar/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:267-353`, `client/OptionInstance.java:543-623` ve `util/Mth.java:344-349,558-560` incelendi. Oracle, gerçek Java value set nesneleri üzerinde doğrulama ve slider dönüşümlerini çalıştırır.

- `IntRange`, min/max uçlarını kapsayan bir aralık doğrular. `next` ve `previous` doğrudan Java i32 toplama/çıkarma yollarıdır; aralık dışına çıkabilir ve i32 taşması sarılır.
- `toSliderValue`, min için `0.0`, max için `1.0` döndürür; diğer değerleri yarım hücre ofsetiyle `Mth.map` yolundan geçirir.
- `fromSliderValue`, slider `1.0` veya üstündeyse Java f32 `0.99999` değerini kullanır; ardından map sonucunu `floor` ile i32'ye çevirir. NaN, sonsuzluk ve aralık dışı slider değerleri bu dönüşüm yolunun parçasıdır.
- `UnitDouble`, `0.0 <= value <= 1.0` koşulunu doğrular. Slider'a çevirme ve slider'dan dönme değeri bit düzeyinde değiştirmez; NaN doğrulamadan geçmez.

Rust karşılığı `src/option.rs` içindeki `IntRange` ve `UnitDouble` tipleridir. `IntRange::with_apply_value_immediately`, Java record alanını taşır; codec, xmap ve GUI/widget yöntemleri bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "option_values"`, IntRange uçlarını/uygulama bayrağını, integer değerlerini, slider f64 bitlerini ve UnitDouble f64 bitlerini taşır. Sonuç doğrulama, next/previous, slider dönüşümü ve UnitDouble kimlik yollarını f64 bitleriyle taşır.
