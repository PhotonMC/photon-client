# Mth smoothstep ve smoothstepDerivative davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `smoothstep(double)` ve `smoothstepDerivative(double)` yordamlarını kapsar.

`smoothstep`, `x * x * x * (x * (x * 6.0 - 15.0) + 10.0)` işlem sırasını korur. `smoothstepDerivative`, `30.0 * x * x * (x - 1.0) * (x - 1.0)` ifadesini aynı sırayla yürütür. Girdi aralığı clamp edilmez; taşma, signed zero ve özel kayan nokta sonuçları Java f64 aritmetiğinden gelir.

Behavior runner `x` alanını on altı haneli lowercase raw f64 bit string'i olarak taşır. Oracle, Java `Mth.smoothstep` veya `Mth.smoothstepDerivative` çağrısını doğrudan yürütür ve sonucu `result.bits` altında raw f64 bit string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:592-598`.

Kapsam dışı kalanlar: `catmullrom`, `sign`, diğer `Mth` interpolasyon yardımcıları ve bu yordamları kullanan istemci akışları.
