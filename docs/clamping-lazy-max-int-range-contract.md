# P05.a — OptionInstance ClampingLazyMaxIntRange sözleşmesi

Bu dilim, üst sınırı çalışma zamanında okunan ve cycle button listesi üreten integer option value set davranışını kapsar. Codec doğrulaması, genel `OptionInstance` value taşıması, GUI slider/callback akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:188-222`, `client/Options.java:908-920`, `client/gui/screens/options/VideoSettingsScreen.java:254-270` ve `util/Mth.java:94-96,344-349,558-560` incelendi. Oracle gerçek `OptionInstance.ClampingLazyMaxIntRange` ve `CycleButton.ValueListSupplier` yollarını çalıştırır.

- `maxInclusive` supplier'ı her erişimde çalışır; testte ilk iki erişim farklı üst sınır döndürür, sonraki erişimler son değeri korur.
- `validateValue`, verilen değeri `[minInclusive, maxInclusive]` aralığına `Mth.clamp` ile sıkıştırır.
- `next` ve `previous`, `IntRangeBase` varsayılanları gibi i32 toplama/çıkarma yollarıdır; sınırda otomatik sıkıştırma yapmaz.
- Slider dönüşümleri `IntRangeBase` varsayılanlarıyla aynı yarım hücre, `floor` ve Java f32 `0.99999` davranışını kullanır.
- `createCycleButton` true döner. `valueListSupplier`, dinamik üst sınır ile `[minInclusive, maxInclusive)` integer listesini üretir ve sıralı immutable snapshot sunar.

Rust karşılığı `src/option.rs` içindeki `ClampingLazyMaxIntRange<Max>` tipidir. Üst sınır generic closure olarak tutulur; codec ve gerçek pencere ölçeği hesabı bu dilime dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "clamping_lazy_max_int_range"`, min/encodable üst sınırlarını, ardışık dinamik üst sınırları, integer probe değerlerini ve slider f64 bitlerini taşır. Sonuç üst sınır erişimlerini, clamp/next/previous/slider davranışını, cycle button listelerini ve slider dönüşlerini taşır.
