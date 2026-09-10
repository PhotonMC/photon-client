# Mth lerp davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki scalar `lerp` f32 ve f64 overload'larını kapsar.

Her overload, `p1 - p0` farkını kendi kayan nokta hassasiyetinde hesaplar, farkı `alpha` ile çarpar ve sonucu `p0` ile toplar. Alpha clamp edilmez; extrapolation, ters uçlar, taşma ve özel değer sonuçları Java aritmetik sırasından gelir.

Behavior runner `alpha`, `p0` ve `p1` alanlarını ilgili hassasiyetin sekiz veya on altı haneli lowercase raw IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.lerp` scalar overload'unu doğrudan çağırır ve sonucu `result.bits` altında aynı hassasiyetin raw bit string'i olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:550-560`.

Kapsam dışı kalanlar: `lerpInt`, `lerpDiscrete`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ı kullanan istemci akışları.
