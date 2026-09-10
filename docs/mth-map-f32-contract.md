# Mth map(float) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `map(float, float, float, float, float)` yordamını kapsar.

Yordam kaynakta `inverseLerp(value, fromMin, fromMax)` sonucunu clamp uygulamadan `lerp(..., toMin, toMax)` yordamına verir. F32 çıkarma, bölme, hedef farkı, çarpma ve toplama Java kaynak sırasını korur; kaynak aralığı ters çevrilmez ve değer kaynak aralığının dışındaysa sonuç extrapolation olarak kalır.

Behavior runner `value`, `from_min`, `from_max`, `to_min` ve `to_max` alanlarını lowercase raw f32 IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.map(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:666-668`.

Kapsam dışı kalanlar: `map(double)`, `clampedMap`, `lerp`, `inverseLerp` ve diğer Mth yardımcıları.
