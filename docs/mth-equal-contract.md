# Mth equal davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `equal(float, float)` ve `equal(double, double)` yollarını kapsar.

Her overload `Math.abs(b - a) < 1.0E-5F` ifadesini aynı hassasiyette yürütür. Karşılaştırma strict'tir: eşik değerindeki fark eşit sayılmaz. Double overload'unda `F` sonekli eşik önce f32 olarak temsil edilir, sonra double'a yükseltilir. Signed zero, subnormal, Infinity ve NaN sonuçları bu değerlendirme sırasının parçasıdır.

Senaryo girdileri raw f32 veya f64 bit string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.equal` çağrısını yürütür ve sonucu boolean olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:158-164`.

Kapsam dışı kalanlar: `Mth.absMax`, positiveModulo, clamp/lerp yardımcıları, genel Vec2/Vec3 equality yolları ve bu yardımcıları kullanan istemci çağrı zincirleri.
