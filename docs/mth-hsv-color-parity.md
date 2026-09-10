# P01.a — Mth HSV renk dönüşümü doğrulaması

`src/math.rs` içindeki `mth_hsv_to_rgb_f32` ve `mth_hsv_to_argb_f32`, Java `Mth.hsvToRgb(float, float, float)` ve `Mth.hsvToArgb(float, float, float, int)` yollarını JSONL adapter'a bağlar. Hue sektör seçimi, f32 ara değerleri, Java int daraltma/clamp sırası ve ARGB paketlemesi Java oracle'ın signed i32 sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Altı hue sektörü, sektör sınırı, `hsvToRgb` alpha sıfır yönlendirmesi, hue sarımı, küçük negatif hue, negatif sektör hatası, sıfır doygunluk, aralık dışı bileşenler, signed zero, NaN ve sonsuzluk yolları için 42 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2270 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2270/2270 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2772/2772 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `89d159ae1a7102bf99ab1e16a0ee38307e541a1e55f0ce683170c77f383d59b7`. Birleşik referans kimliği: `d90dcaddb9651f596532bebab20189c338c387b0754412659b0ead41a64b0e38`.

Adapter hashleri: debug `e266939af158c95ffa41256e20225b5f4ab874dc4968a4583331fa3f8b384932`, release `145e9205a5ebbb66fd7c323df8114890c98a626842ee8b1b6a9f5903575fe1bc`.

Yerel raporlar: `compare-tyhl4f5t` ve `compare-wq9tcf1l` (math debug/release), `compare-kxl8epyn` ve `compare-wa8tju9p` (birleşik debug/release), `verify-p0yowjhd` (math) ve `verify-34qu3nzb` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth HSV yordamları, ARGB renk paketlemesi ve bu dilimin 42 raw f32/i32 girdisiyle sınırlıdır. Diğer `ARGB` yardımcıları, renk uzayı dönüşümleri, gamma/linear dönüşümleri ve renderer renk kullanımları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
