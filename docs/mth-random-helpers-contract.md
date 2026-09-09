# Mth random helper davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `randomBetweenInclusive(RandomSource, int, int)`, `randomBetween(RandomSource, float, float)` ve `normal(RandomSource, float, float)` yollarını kapsar.

`randomBetweenInclusive`, `maxInclusive - min + 1` Java i32 aritmetiğiyle bound oluşturur, `RandomSource.nextInt(bound)` çağrısını yapar ve min değerini ekler. Eşit sınırlar da kısa devre edilmez; `nextInt(1)` çekişi seed ilerlemesine yansır. `randomBetween`, `random.nextFloat() * (maxExclusive - min) + min` sırasını f32 olarak yürütür. `normal`, `mean + (float)random.nextGaussian() * deviation` sırasını korur. Aralık veya Gaussian sonuçları signed zero, Infinity ve NaN üretebilir.

Her senaryoda RandomSource sabit seed ile yeniden oluşturulur. Oracle helper sonucundan sonra kontrollü bir takip çekişi yapar: inclusive için `nextInt`, between için `nextFloat`, normal için `nextGaussian`. Böylece bound tüketimi ve Gaussian cache davranışı sonuç sözleşmesine dahil edilir. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki Mth çağrısını yürütür; Rust adaptörü aynı JSON bit/değer sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:690-700`.

Kapsam dışı kalanlar: `Mth.nextInt/nextFloat/nextDouble`, RandomSource'un diğer implementasyonları, overflow ile geçersiz inclusive bound istisnaları ve bu yardımcıları kullanan istemci çağrı zincirleri.
