# Mth square(long) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(long)` yordamını kapsar.

Yordam kaynakta tek bir i64 çarpımıyla `x * x` sonucunu döndürür. Java long aritmetiğinin 64 bit iki'nin tümleyeni sarması korunur; Rust tarafında genişletme, clamp veya tanımsız taşma kullanılmaz.

Behavior runner `x` alanını signed decimal i64 string olarak taşır. Oracle, Java `Mth.square(long)` overload'unu doğrudan çağırır ve sonucu `result.value` altında signed decimal i64 string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:650-652`.

Kapsam dışı kalanlar: `square(float)`, `square(double)`, `square(int)`, `cube`, `clampedMap`, `map` ve diğer Mth yardımcıları.
