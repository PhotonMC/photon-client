# Mth positiveModulo davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `positiveModulo(float, float)` ve `positiveModulo(double, double)` yollarını kapsar.

Her overload aynı hassasiyette `(input % mod + mod) % mod` sırasını izler. Bu ifade pozitif modül için negatif girdileri `[0, mod)` aralığına taşır; negatif modül, tam kat, signed zero, subnormal, sıfır, Infinity ve NaN durumlarında ara remainder ve toplama sonuçları gözlenebilir sözleşmenin parçasıdır. Rust uygulaması matematiksel eşdeğer bir modulo formülüne dönüştürülmez.

Senaryo girdileri raw f32 veya f64 bit string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.positiveModulo` çağrısını yürütür ve sonucu raw f32/f64 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:166-176`.

Kapsam dışı kalanlar: integer `positiveModulo`, `Mth.equal`, `absMax`, clamp/lerp yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri.
