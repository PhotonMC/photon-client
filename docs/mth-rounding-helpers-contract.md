# Mth yuvarlama yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `ceil(float)`, `ceil(double)`, `lfloor(double)` ve `ceilLong(double)` yollarını kapsar.

`ceil(float)` ve `ceil(double)`, Java `Math.ceil` sonucunu sırasıyla int'e daraltır. `lfloor(double)`, `Math.floor` sonucunu long'a; `ceilLong(double)`, `Math.ceil` sonucunu long'a daraltır. NaN sıfıra, hedef tür aralığı dışındaki pozitif değerler üst sınıra, negatif değerler alt sınıra daraltılır. Kesirli değerler için ceil ve floor yönü, signed zero girdisi ve özel değer sonuçları gözlenebilir sözleşmenin parçasıdır.

Senaryo girdileri raw f32 veya f64 bit string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu signed int veya long decimal string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:62-92` (`floor`, `lfloor`, `ceil`, `ceilLong`).

Kapsam dışı kalanlar: mevcut `Mth.floor` adaptörleri, floating-point `clamp`/`wrapDegrees` overload'ları, `Mth.sqrt`, diğer dönüşüm yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri.
