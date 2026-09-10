# Mth square(float) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(float)` yordamını kapsar.

Yordam tek bir f32 çarpımıyla `x * x` sonucunu döndürür. Çarpım kaynakta görünen sırayla yapılır; Rust tarafında ortak bir `powi`, f64 genişletmesi veya özel değer normalizasyonu kullanılmaz. Signed zero karesi pozitif sıfırdır; sonlu taşma Infinity, alt taşma ve subnormal yuvarlama Java f32 çarpma semantiğine bırakılır.

Behavior runner `x` alanını lowercase raw f32 IEEE-754 bit dizgisi olarak taşır. Oracle, Java `Mth.square(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgisi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:634-636`.

Kapsam dışı kalanlar: `square(double)`, `square(int)`, `square(long)`, `cube`, `triangleWave`, `clampedMap`, `map` ve diğer Mth yardımcıları.
