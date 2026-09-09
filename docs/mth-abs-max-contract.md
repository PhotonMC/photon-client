# Mth absMax davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `absMax(float, float)` ve `absMax(double, double)` yollarını kapsar.

Her overload iki girdiye sırasıyla mutlak değer uygular ve sonuçları aynı hassasiyette `Math.max` ile karşılaştırır. Signed zero, subnormal, sonlu sınır, Infinity ve NaN sonuçları Java `Math.abs`/`Math.max` semantiğinin parçasıdır; geometrik veya tamsayı `absMax` yollarıyla birleştirilmez.

Senaryo girdileri raw f32 veya f64 bit string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.absMax` çağrısını yürütür ve sonucu raw f32/f64 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:126-136`.

Kapsam dışı kalanlar: integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri.
