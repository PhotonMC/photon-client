# Mth rotLerpRad davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotLerpRad(float, float, float)` yordamını kapsar.

Yordam önce `to - from` farkını f32 olarak hesaplar. Fark `-π` değerinden küçük kaldığı sürece f32 `2π` ekler; fark `π` değerine eşit veya büyük olduğu sürece aynı f32 `2π` değerini çıkarır. Sonuç, clamp edilmemiş `alpha` ile `from + alpha * fark` kaynak sırasıyla üretilir. `+π` sınırı `-π` yönüne taşınır; `-π` sınırı olduğu gibi kalır. Signed zero, taşma ve NaN ara sonuçları bu işlemlerin Java davranışıyla birlikte korunur.

Behavior runner `alpha`, `from` ve `to` alanlarını lowercase raw f32 IEEE-754 bit dizgeleri olarak taşır. Oracle, Java `Mth.rotLerpRad` yordamını doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgesi olarak döndürür; beklenen değerler Rust'tan türetilmez. Özgün senaryolar sonlu farkları ve NaN yollarını kapsar. Farkın sonsuz olması Java'nın sonlanmayan `while` döngülerine girdiğinden, Infinity farkı üreten girdiler oracle senaryolarına dahil edilmemiştir.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:616-630`.

Kapsam dışı kalanlar: `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları.
