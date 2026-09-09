# Mth clampedLerp davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `clampedLerp(float, float, float)` ve `clampedLerp(double, double, double)` yollarını kapsar.

Her overload faktörü önce iki strict koşulla değerlendirir: faktör sıfırdan küçükse min, birden büyükse max döner. Diğer durumda `min + factor * (max - min)` sırasındaki lerp hesabı yürütülür. Bu nedenle `NaN` faktörü clamp dallarını atlayıp lerp hesabına girer; signed zero, ters aralık, taşma, Infinity ve NaN endpoint sonuçları işlem sırasının parçasıdır.

Senaryo girdileri raw f32 veya f64 bit string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.clampedLerp` çağrısını yürütür ve sonucu bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:94-124` ve `net/minecraft/util/Mth.java:550-560`.

Kapsam dışı kalanlar: `Mth.clamp` overload'ları, bağımsız `Mth.lerp` overload'ları, `inverseLerp`, Vec3 lerp ve bu yardımcıları kullanan istemci çağrı zincirleri.
