# P05.a — OptionInstance SliderableEnum sözleşmesi

Bu dilim, ayar widget'ı veya seçenek dosyası gerektirmeyen sıralı enum value set davranışını kapsar. Genel `OptionInstance` value taşıması, Codec JSON serileştirmesi ve GUI callback akışı sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:484-522`, `client/Options.java:157-165`, `client/GraphicsPreset.java:12-20` ve `util/Mth.java:344-349,558-560` incelendi. Oracle gerçek `OptionInstance.SliderableEnum<GraphicsPreset>` nesnesini çalıştırır.

- Değer sırası `GraphicsPreset.values()` sırasıdır: `FAST`, `FANCY`, `FABULOUS`, `CUSTOM`.
- İlk değer slider `0.0`, son değer slider `1.0` alır. Diğer değerler enum indeksini `Mth.map` ile `[0, 1]` aralığına taşır.
- `next` ve `previous` mevcut indeksin bir üstünü veya altını seçer; uçlarda indeks `[0, values.size() - 1]` içinde tutulur.
- `fromSliderValue`, slider `1.0` veya üstündeyse Java f32 `0.99999` değerini kullanır. Ardından `Mth.map` sonucu `floor` edilir ve enum indeksine sıkıştırılır. Negatif, sonsuz ve NaN slider değerleri aynı Java dönüşüm yolundadır.
- `validateValue`, değerin value listesinde bulunup bulunmadığını döndürür.

Rust karşılığı `src/option.rs` içindeki `SliderableEnum<T>` tipidir. Value listesi boş olamaz; Java'da bu record ile oluşturulabilse de slider metotları boş listede ilk/son değere eriştiği için bu dilim istemcide kullanılan geçerli enum listelerini modeller.

## JSONL sınırı

İstek `family: "math"`, `operation: "sliderable_enum"`, probe enum adlarını ve slider f64 bitlerini taşır. Sonuç value listesi, her probe için doğrulama/slider/komşu enum değerlerini ve her slider için `fromSliderValue` enum sonucunu taşır.
