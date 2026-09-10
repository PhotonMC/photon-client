# Mth square(double) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(double)` yordamını kapsar.

Yordam tek bir f64 çarpımıyla `x * x` sonucunu döndürür. Çarpım kaynakta görünen sırayla yapılır; Rust tarafında `powi`, f32 daraltması veya özel değer normalizasyonu kullanılmaz. Signed zero karesi pozitif sıfırdır; alt taşma, subnormal yuvarlama ve sonlu taşma Java f64 çarpma semantiğine bırakılır.

Behavior runner `x` alanını lowercase raw f64 IEEE-754 bit dizgisi olarak taşır. Oracle, Java `Mth.square(double)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f64 bit dizgisi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:642-644`.

Kapsam dışı kalanlar: `square(float)`, `square(int)`, `square(long)`, `cube`, `clampedMap`, `map` ve diğer Mth yardımcıları.
