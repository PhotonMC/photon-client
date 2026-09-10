# Mth cube(float) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `cube(float)` yordamını kapsar.

Yordam `x * x * x` ifadesinin kaynakta görünen, soldan birleşimli f32 çarpımlarını döndürür. İlk çarpımın ara sonucu f32 olarak yuvarlanır ve bu sonuç tekrar `x` ile çarpılır; f64 genişletmesi, `powi` veya özel değer normalizasyonu yapılmaz. Signed zero, subnormal yuvarlama ve sonlu taşma Java f32 çarpma semantiğine bırakılır.

Behavior runner `x` alanını lowercase raw f32 IEEE-754 bit dizgisi olarak taşır. Oracle, Java `Mth.cube(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgisi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:638-640`.

Kapsam dışı kalanlar: `square` overload'ları, `square(double)`, `square(int)`, `square(long)`, `clampedMap`, `map` ve diğer Mth yardımcıları.
