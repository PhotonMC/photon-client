# Mth mulAndTruncate davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `mulAndTruncate(Fraction, int)` helper'ını kapsar. `Fraction`, sabitlenen runtime'daki Apache Commons Lang 3.20.0 sınıfıdır.

Helper, Fraction'ın numerator alanını factor ile Java i32 çarpması olarak çarpar ve sonucu denominator alanına Java integer bölmesiyle böler. Çarpma taşması i32 genişliğinde sarılır; bölme sıfıra doğru kesilir. Java'nın `Integer.MIN_VALUE / -1` özel sonucu yine `Integer.MIN_VALUE` olarak korunur.

Behavior runner girdileri numerator, denominator ve factor alanlarını signed decimal i32 olarak taşır. Oracle, geçerli senaryolarda pozitif ve sıfır olmayan denominator ile `Fraction.getFraction(numerator, denominator)` oluşturur; böylece helper'a ulaşan Fraction alanları açıkça sabitlenir. Sonuç `value` altında signed decimal i32 olarak taşınır. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:767`; Fraction bağımlılığı `org.apache.commons.lang3.math.Fraction`.

Kapsam dışı kalanlar: Fraction oluşturma hataları ve diğer factory overload'ları, Fraction aritmetiği, `mulAndTruncate` çağıran bundle/tooltip akışları ve quaternion helper'ları.
