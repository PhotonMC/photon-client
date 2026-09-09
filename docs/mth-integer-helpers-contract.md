# Mth tamsayı yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` yollarını kapsar. Her senaryoda bölen sıfır değildir; Java'nın sıfıra bölme istisnası bu dilimin çıktısı değildir.

`floorDiv`, bölüm ve kalan işaretleri farklıysa sıfıra doğru Java bölümünü bir azaltarak matematiksel tabana yuvarlar. `positiveModulo`, aynı `floorDiv` sonucunu kullanarak `input - floorDiv(input, divisor) * divisor` sözleşmesini ve bölenin işaretini korur. `Integer.MIN_VALUE / -1` gibi Java tamsayı taşmaları iki tamamlayıcılı genişlikte sarılır.

`positiveCeilDiv`, Java kaynağındaki `-Math.floorDiv(-input, divisor)` sırasını korur. Özellikle minimum değerin unary negation işlemi sarılır. `roundToward`, bu sonucu `multiple` ile Java int/long çarpması olarak tekrar sarar; sonuç temsil edilebilir aralığa sığmasa da hata üretmez.

Senaryo girdileri tamsayıları decimal string olarak taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu string olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:142-144` (`floorDiv`), `:164-166` (`positiveModulo`), `:674-687` (`roundToward` ve `positiveCeilDiv`).

Kapsam dışı kalanlar: sıfıra bölme istisnasının ayrıntılı hata nesnesi, `Mth`'nin diğer tamsayı overload'ları, float/double yardımcıları ve istemci algoritmalarının bu yardımcıları hangi çağrı zincirinde kullandığı.
