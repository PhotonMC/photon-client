# Mth sign davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `sign(double)` yordamını kapsar.

`sign`, önce girdiyi `0.0` ile eşitlik açısından sınar ve eşitse `0` döndürür. Eşit değilse yalnızca `number > 0.0` koşulunda `1`, kalan durumda `-1` döndürür. Böylece `-0.0` ve `+0.0` aynı sıfır dalına gider; NaN ise iki koşulu da geçemediği için `-1` olur.

Behavior runner `x` alanını on altı haneli lowercase raw f64 bit string'i olarak taşır. Oracle, Java `Mth.sign` çağrısını doğrudan yürütür ve sonucu `result.value` altında signed decimal string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:600-606`.

Kapsam dışı kalanlar: `rotLerp`, `rotLerpRad`, `triangleWave`, diğer açı yardımcıları ve `sign` kullanan istemci akışları.
