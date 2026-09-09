# Mth tamsayı temel yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` yollarını kapsar. `isMultipleOf` senaryolarında bölen sıfır değildir; Java'nın sıfıra bölme istisnası bu dilimin çıktısı değildir.

`abs(int)`, Java `Math.abs` davranışını izler; `Integer.MIN_VALUE` için temsil edilemeyen pozitif değer iki tamamlayıcılı int genişliğinde yine `Integer.MIN_VALUE` olur. `clamp` önce değeri alt sınıra göre `Math.max`, sonra üst sınıra göre `Math.min` ile sınırlar. Sınırların ters verilmesi hata üretmez; iki `Math` çağrısının sırası korunur.

`isMultipleOf`, signed int kalanını sıfırla karşılaştırır. Java'nın `Integer.MIN_VALUE % -1` kenarı da sarılı sonuç olan sıfırı verir. `wrapDegrees(int)` önce `angle % 360` ile Java signed kalanını alır; `180` ve üstünü 360 azaltır, `-180` altını 360 artırır ve sonucu `[-180, 180)` aralığına taşır.

Senaryo girdileri tamsayıları decimal string olarak taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu string veya boolean olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:74-100` (`abs`, `clamp`), `:178-202` (`isMultipleOf`, `wrapDegrees(int)`).

Kapsam dışı kalanlar: sıfıra bölme istisnasının ayrıntılı hata nesnesi, `wrapDegrees` long/float/double overload'ları, `packDegrees`/`unpackDegrees`, float/double clamp ve diğer Mth yardımcıları.
