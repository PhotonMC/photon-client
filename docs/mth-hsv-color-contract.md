# Mth HSV renk dönüşümü davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `hsvToRgb(float, float, float)` ve `hsvToArgb(float, float, float, int)` yordamlarını kapsar. `hsvToRgb` aynı dönüşümü alpha değeri `0` ile çağırır.

`hue * 6.0F`, `f`, `p`, `q` ve `t` ara değerleri Java f32 hassasiyetinde kaynak sırasıyla hesaplanır. Hue'nun sıfıra doğru Java int'e daraltılmış değeri signed `% 6` işleminden geçirilerek altı renk sektöründen biri seçilir. Negatif sektör için kaynak `RuntimeException`'ı adapter'da `outcome: error` ve `error: hue_out_of_range` olarak taşınır.

Renk bileşenleri `component * 255.0F` sonrasında Java float-to-int daraltmasıyla i32'e çevrilir ve `Mth.clamp(int, 0, 255)` ile sınırlandırılır. `ARGB.color` alpha ve üç renk kanalının yalnızca düşük sekiz bitini ilgili ARGB i32 bit düzenine yerleştirir; sonuç signed decimal string olarak taşınır.

Java oracle `Mth.hsvToRgb` ve `Mth.hsvToArgb` yordamlarını doğrudan çağırır; beklenen sonuçlar Rust uygulamasından türetilmez. Senaryolar altı sektör sınırını, hue sarımını, küçük negatif/negatif sektör hatasını, sıfır doygunluğu, aralık dışı bileşenleri, signed zero, NaN ve sonsuzluk yollarını kapsar.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:465-513` ve `net/minecraft/util/ARGB.java:68-70`.

Kapsam dışı kalanlar: diğer `ARGB` bileşen yardımcıları, renk uzayı dönüşümleri, gamma/linear dönüşümleri ve renderer renk kullanımları.
