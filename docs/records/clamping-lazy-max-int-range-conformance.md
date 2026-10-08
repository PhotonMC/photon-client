# P05.a — OptionInstance ClampingLazyMaxIntRange sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, üst sınırı çalışma zamanında okunan ve cycle button listesi üreten integer option value set davranışını kapsar. Codec doğrulaması, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:188-222`, `client/Options.java:908-920`, `client/gui/screens/options/VideoSettingsScreen.java:254-270` ve `util/Mth.java:94-96,344-349,558-560` incelendi. Oracle gerçek `OptionInstance.ClampingLazyMaxIntRange` ve `CycleButton.ValueListSupplier` yollarını çalıştırır.

- `maxInclusive` supplier'ı her erişimde çalışır; testte ilk iki erişim farklı üst sınır döndürür, sonraki erişimler son değeri korur.
- `validateValue`, verilen değeri `[minInclusive, maxInclusive]` aralığına `Mth.clamp` ile sıkıştırır.
- `next` ve `previous`, `IntRangeBase` varsayılanları gibi i32 toplama/çıkarma yollarıdır; sınırda otomatik sıkıştırma yapmaz.
- Slider dönüşümleri `IntRangeBase` varsayılanlarıyla aynı yarım hücre, `floor` ve Java f32 `0.99999` davranışını kullanır.
- `createCycleButton` true döner. `valueListSupplier`, dinamik üst sınır ile `[minInclusive, maxInclusive)` integer listesini üretir ve sıralı immutable snapshot sunar.

Rust karşılığı `crates/photon-game/src/options/values.rs` içindeki `ClampingLazyMaxIntRange<Max>` tipidir. Üst sınır generic closure olarak tutulur; codec ve gerçek pencere ölçeği hesabı bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "clamping_lazy_max_int_range"`, min/encodable üst sınırlarını, ardışık dinamik üst sınırları, integer probe değerlerini ve slider f64 bitlerini taşır. Sonuç üst sınır erişimlerini, clamp/next/previous/slider davranışını, cycle button listelerini ve slider dönüşlerini taşır.


## Client-parity doğrulaması

`crates/photon-game/src/options/values.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.ClampingLazyMaxIntRange` davranışını uygular. Java oracle gerçek dinamik max supplier'ı, inherited `IntRangeBase` yollarını ve `CycleButton.ValueListSupplier` listesini çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `a609ada324e896ae42aa8fc7f01c20b288289aede5d9e0b1687c3fa5cd4c4f60`; tam corpus referansı `18d1766f46ca8c601275e6c300f05426f971447bf83491eeba96fba4be6b568e` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-3e2fz9jh` ve `compare-ugv8j904`; tam debug/release karşılaştırmaları `compare-v_srm0lz` ve `compare-3iazkjm7`; Java tekrar doğrulaması `verify-kwmjb6el` raporlarında fark üretmedi. Senaryolar dinamik üst sınır erişimini, clamp edilen değerleri, i32 next/previous, slider sınırlarını, özel slider değerlerini ve cycle button listesini kapsar.

Codec, gerçek `Window.calculateScale`, VideoSettingsScreen olayları, option callback'leri ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
