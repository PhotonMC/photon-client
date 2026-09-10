# Mth length davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `lengthSquared(double, double)`, `length(double, double)`, `length(float, float)`, `lengthSquared(double, double, double)`, `length(double, double, double)` ve `lengthSquared(float, float, float)` yollarını kapsar.

İki bileşenli double `lengthSquared`, kareleri `x * x + y * y` kaynak sırasıyla toplar. Double `length` bu overload'un sonucuna `Math.sqrt` uygular. `length(float, float)` çağrısı da kaynak içindeki overload çözümlemesiyle double `lengthSquared` yoluna gider ve karekökü f32'ye daraltır. Üç bileşenli double overload'ları aynı işlemi `x`, `y`, `z` sırasıyla yürütür. Üçlü f32 `lengthSquared`, her çarpma ve toplama ara sonucunu f32 hassasiyetinde tutar.

Her senaryo girdileri overload'un gerçek parametre hassasiyetinde raw f32 veya f64 bit string'leri taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki Mth çağrısını yürütür ve sonucu aynı hassasiyetin raw bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Signed zero, overflow, Infinity ve NaN sonuçları sözleşmeye dahildir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:702-726`.

Kapsam dışı kalanlar: Vec2/Vec3 `length` ve `lengthSqr` metotları, `Mth.quantize`, `Mth.outFromOrigin` ve bu yardımcıları kullanan istemci çağrı zincirleri.
