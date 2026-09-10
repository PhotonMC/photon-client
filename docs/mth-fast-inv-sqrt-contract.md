# Mth fastInvSqrt davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `fastInvSqrt(double)` yordamını kapsar. Yordam f64 girdinin raw bitlerini signed long görünümüne alır, sabit bit tahmini üretir ve tek Newton düzeltmesini Java kaynak sırasıyla uygular.

`Double.doubleToRawLongBits` NaN payload'ını korur; `i >> 1` signed long aritmetik sağa kaydırmadır ve sabit çıkarma long genişliğinde değerlendirilir. Sonraki çarpma/çıkarma işlemleri f64'tür. Çıktı raw f64 bit string olarak taşınır.

Java oracle `Mth.fastInvSqrt` yordamını doğrudan çağırır; beklenen bitler Rust uygulamasından türetilmez. Senaryolar sıfır, signed zero, pozitif/negatif normal ve subnormal değerler, sınırlar, sonsuzluk ve NaN payload'larını kapsar.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:449-455`.

Kapsam dışı kalanlar: `atan2`, JOML `invSqrt`, `fastInvCubeRoot` ve gerçek ray/vektör kullanım sıraları.
