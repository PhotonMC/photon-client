# Mth HSV renk dönüşümü davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `hsvToRgb(float, float, float)` ve `hsvToArgb(float, float, float, int)` yordamlarını kapsar. `hsvToRgb` aynı dönüşümü alpha değeri `0` ile çağırır.

`hue * 6.0F`, `f`, `p`, `q` ve `t` ara değerleri Java f32 hassasiyetinde kaynak sırasıyla hesaplanır. Hue'nun sıfıra doğru Java int'e daraltılmış değeri signed `% 6` işleminden geçirilerek altı renk sektöründen biri seçilir. Negatif sektör için kaynak `RuntimeException`'ı adapter'da `outcome: error` ve `error: hue_out_of_range` olarak taşınır.

Renk bileşenleri `component * 255.0F` sonrasında Java float-to-int daraltmasıyla i32'e çevrilir ve `Mth.clamp(int, 0, 255)` ile sınırlandırılır. `ARGB.color` alpha ve üç renk kanalının yalnızca düşük sekiz bitini ilgili ARGB i32 bit düzenine yerleştirir; sonuç signed decimal string olarak taşınır.

Java oracle `Mth.hsvToRgb` ve `Mth.hsvToArgb` yordamlarını doğrudan çağırır; beklenen sonuçlar Rust uygulamasından türetilmez. Senaryolar altı sektör sınırını, hue sarımını, küçük negatif/negatif sektör hatasını, sıfır doygunluğu, aralık dışı bileşenleri, signed zero, NaN ve sonsuzluk yollarını kapsar.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:465-513` ve `net/minecraft/util/ARGB.java:68-70`.

Kapsam dışı kalanlar: diğer `ARGB` bileşen yardımcıları, renk uzayı dönüşümleri, gamma/linear dönüşümleri ve renderer renk kullanımları.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `mth_hsv_to_rgb_f32` ve `mth_hsv_to_argb_f32`, Java `Mth.hsvToRgb(float, float, float)` ve `Mth.hsvToArgb(float, float, float, int)` yollarını JSONL adapter'a bağlar. Hue sektör seçimi, f32 ara değerleri, Java int daraltma/clamp sırası ve ARGB paketlemesi Java oracle'ın signed i32 sonucu ile karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Altı hue sektörü, sektör sınırı, `hsvToRgb` alpha sıfır yönlendirmesi, hue sarımı, küçük negatif hue, negatif sektör hatası, sıfır doygunluk, aralık dışı bileşenler, signed zero, NaN ve sonsuzluk yolları için 42 özgün senaryo hazırlandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `89d159ae1a7102bf99ab1e16a0ee38307e541a1e55f0ce683170c77f383d59b7`. Birleşik referans kimliği: `d90dcaddb9651f596532bebab20189c338c387b0754412659b0ead41a64b0e38`.

Adapter hashleri: debug `e266939af158c95ffa41256e20225b5f4ab874dc4968a4583331fa3f8b384932`, release `145e9205a5ebbb66fd7c323df8114890c98a626842ee8b1b6a9f5903575fe1bc`.

Yerel raporlar: `compare-tyhl4f5t` ve `compare-wq9tcf1l` (math debug/release), `compare-kxl8epyn` ve `compare-wa8tju9p` (birleşik debug/release), `verify-p0yowjhd` (math) ve `verify-34qu3nzb` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth HSV yordamları, ARGB renk paketlemesi ve bu dilimin 42 raw f32/i32 girdisiyle sınırlıdır. Diğer `ARGB` yardımcıları, renk uzayı dönüşümleri, gamma/linear dönüşümleri ve renderer renk kullanımları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
