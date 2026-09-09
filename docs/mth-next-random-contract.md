# Mth next random davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `nextInt(RandomSource, int, int)`, `nextFloat(RandomSource, float, float)` ve `nextDouble(RandomSource, double, double)` yollarını kapsar.

Int overload'u minInclusive değeri maxInclusive değerinden küçük değilse minInclusive döndürür. Aksi halde inclusive aralık genişliğini `maxInclusive - minInclusive + 1` Java i32 aritmetiğiyle kurar, RandomSource'un bound'lu çekişine verir ve minInclusive değerini ekler. Float/double overload'ları min değeri max değerinden küçük değilse min'i döndürür; diğer durumda kendi hassasiyetlerindeki `random.next*() * (max - min) + min` sırasını yürütür. Karşılaştırmalar strict Java floating-point kurallarını kullanır; signed zero, Infinity ve NaN dalları korunur.

Her senaryoda RandomSource sabit seed ile yeniden oluşturulur. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki Mth çağrısını yürütür ve sonucu signed decimal veya raw f32/f64 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:142-156`.

Kapsam dışı kalanlar: `Mth.randomBetweenInclusive`, `randomBetween`, `normal`, RandomSource'un diğer implementasyonları, overflow ile geçersiz bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri.
