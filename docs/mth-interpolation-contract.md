# Mth lerp2/lerp3/catmullrom davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `lerp2(double)`, `lerp3(double)` ve `catmullrom(float)` yordamlarını kapsar. Her yordam için 20 sentetik senaryo hazırlanır.

`lerp2`, önce iki x ekseni interpolasyonunu aynı `alpha1` ile hesaplar, ardından bu iki sonucu `alpha2` ile birleştirir. `lerp3`, alt ve üst düzlemlerde `lerp2(alpha1, alpha2, ...)` çağrılarını kaynak sırasıyla yapar ve iki sonucu `alpha3` ile birleştirir. Bu yollar alpha değerlerini clamp etmez; çıkarma, çarpma ve toplama sırası Java scalar `lerp` yordamıyla korunur.

`catmullrom`, dört f32 kontrol noktasını kaynakta yazılan kübik polinomla işler. `alpha * alpha` ve `alpha * alpha * alpha` alt ifadeleri, katsayı çarpımları ve toplamlar Java f32 işlem sırasını korur. Signed zero, subnormal, taşma, Infinity ve NaN sonuçları bu aritmetik yolun parçasıdır.

Behavior runner `lerp2` ve `lerp3` girdilerini `alpha1`, `alpha2`, `alpha3` ve ilgili `x...` alanlarında lowercase raw f64 IEEE-754 bit string'leriyle; `catmullrom` girdilerini `alpha`, `p0`, `p1`, `p2`, `p3` alanlarında raw f32 bit string'leriyle taşır. Java oracle yordamı doğrudan çağırır ve sonucu hassasiyetine göre `result.bits` altında döndürür. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:558-590`.

Kapsam dışı kalanlar: scalar `lerp`, `lerpInt`, `lerpDiscrete`, Vec3 `lerp`, `clampedLerp`, `inverseLerp`, `map`, `wobble` ve bu helper'ları kullanan istemci akışları.
