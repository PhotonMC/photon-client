# Mth lerpInt ve lerpDiscrete davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `lerpInt` ve `lerpDiscrete` overload'larını kapsar.

`lerpInt`, önce `p1 - p0` i32 farkını Java sarmasıyla hesaplar, bu farkı f32 alpha ile çarpar, sonucu `Mth.floor` ile i32'ye daraltır ve `p0` ile toplar. `lerpDiscrete` aynı delta'yı kurar; floor girdisinde delta'dan bir çıkarır ve yalnızca alpha `0.0F` değerinden büyükse sonuca bir ekler. Her iki sonuçta i32 toplama sarması korunur; alpha clamp edilmez.

Behavior runner `alpha` alanını sekiz haneli lowercase raw f32 bit string'i, `p0` ve `p1` alanlarını signed decimal i32 string'leri olarak taşır. Oracle, Java `Mth.lerpInt` veya `Mth.lerpDiscrete` çağrısını doğrudan yürütür ve sonucu `result.value` altında signed decimal string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:541-548`.

Kapsam dışı kalanlar: scalar f32/f64 `lerp`, Vec3 overload'u, `lerp2`, `lerp3`, `catmullrom`, `clampedLerp`, `inverseLerp` ve bu helper'ları kullanan istemci akışları.
