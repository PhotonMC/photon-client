# Mth clampedMap(float) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `clampedMap(float, float, float, float, float)` yordamını kapsar.

Yordam kaynakta `inverseLerp(value, fromMin, fromMax)` sonucunu `clampedLerp(..., toMin, toMax)` yordamına verir. F32 çıkarma, bölme, clamp dalları ve son lerp işlemleri Java kaynak sırasını korur; giriş aralığı ters çevrilmez ve sonuç özel değerlerle normalize edilmez.

Behavior runner `value`, `from_min`, `from_max`, `to_min` ve `to_max` alanlarını lowercase raw f32 IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.clampedMap(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:658-660`.

Kapsam dışı kalanlar: `clampedMap(double)`, `map`, `clampedLerp`, `inverseLerp` ve diğer Mth yardımcıları.
