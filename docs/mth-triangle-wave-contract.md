# Mth triangleWave davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `triangleWave(float, float)` yordamını kapsar.

Yordam `index % period` sonucundan `period * 0.5F` değerini çıkarır, mutlak değeri alır, bundan `period * 0.25F` çıkarır ve aynı f32 `period * 0.25F` değeriyle böler. Bu işlemler Java kaynak sırasıyla yapılır; periyot normalize edilmez veya clamp edilmez. Negatif periyot, sıfır ve NaN/Infinity ara sonuçları Java f32 aritmetiğiyle birlikte korunur.

Behavior runner `index` ve `period` alanlarını lowercase raw f32 IEEE-754 bit dizgeleri olarak taşır. Oracle, Java `Mth.triangleWave` yordamını doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgesi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:630-632`.

Kapsam dışı kalanlar: `square`, `cube`, `clampedMap`, `map`, diğer Mth yardımcıları ve `triangleWave` kullanan istemci akışları.
