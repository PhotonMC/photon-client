# Mth fastInvCubeRoot davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `fastInvCubeRoot(float)` yordamını kapsar. Yordam f32 girdinin `Float.floatToIntBits` sonucunu alır, i32 bit tahmini üretir ve aynı f32 Newton düzeltmesini iki kez uygular.

`Float.floatToIntBits` tüm NaN payload'larını canonical `0x7fc00000` değerine çevirir; diğer bit desenleri korunur. `i / 3` signed Java i32 bölümüdür ve sıfıra doğru kesilir. Bit tahmini `Float.intBitsToFloat` ile f32'ye dönüştükten sonra her çarpma, toplama ve bölme f32 hassasiyetinde kaynak sırasıyla yürür.

Java oracle `Mth.fastInvCubeRoot` yordamını doğrudan çağırır; beklenen bitler Rust uygulamasından türetilmez. Girdiler ve sonuç raw f32 bit string olarak taşınır.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:457-465`.

Kapsam dışı kalanlar: `invSqrt`, `fastInvSqrt`, `atan2`, başka küpkök yaklaşımları ve renk dönüşümleri.
