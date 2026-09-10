# Mth rotLerp davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotLerp(float, float, float)` ve `rotLerp(double, double, double)` overload'larını kapsar.

Her overload, `to - from` farkını kendi hassasiyetinde hesaplar, aynı hassasiyette `wrapDegrees` ile farkı `[-180, 180)` aralığına taşır ve sonucu `from + a * fark` kaynak sırasıyla üretir. `a` clamp edilmez; signed zero, taşma, Infinity ve NaN ara sonuçları bu işlemlerin Java davranışıyla birlikte korunur. `+180` farkı `-180` olur, `-180` farkı olduğu gibi kalır.

Behavior runner `alpha`, `from` ve `to` alanlarını overload hassasiyetine göre lowercase raw IEEE-754 bit dizgeleri olarak taşır. Oracle, Java `Mth.rotLerp` overload'unu doğrudan çağırır ve sonucu `result.bits` altında aynı hassasiyetin raw bit dizgesi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:608-614`.

Bu belgenin kapsamı dışında kalanlar: `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerp` kullanan istemci akışları. `rotLerpRad` ayrı bir sözleşme ve parity belgesinde ele alınır.
