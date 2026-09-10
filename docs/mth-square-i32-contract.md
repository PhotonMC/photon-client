# Mth square(int) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(int)` yordamını kapsar.

Yordam kaynakta tek bir i32 çarpımıyla `x * x` sonucunu döndürür. Java int aritmetiğinin 32 bit iki'nin tümleyeni sarması korunur; Rust tarafında genişletme, clamp veya tanımsız taşma kullanılmaz.

Behavior runner `x` alanını signed decimal i32 string olarak taşır. Oracle, Java `Mth.square(int)` overload'unu doğrudan çağırır ve sonucu `result.value` altında signed decimal i32 string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:646-648`.

Kapsam dışı kalanlar: `square(float)`, `square(double)`, `square(long)`, `cube`, `clampedMap`, `map` ve diğer Mth yardımcıları.
