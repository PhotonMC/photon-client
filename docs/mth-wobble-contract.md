# Mth wobble(double) davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `wobble(double)` yordamını kapsar. 20 sentetik senaryo, koordinatın sabit seed türetmesini, tek çekişli gürültüyü, sınır değerlerini ve özel f64 girdileri kapsar.

Yordam önce `coord * 3000.0` çarpımını yapar ve `Mth.floor(double)` sonucunu `int` seed olarak kullanır. Bu seed ile `RandomSource.createThreadLocalInstance(long)` üzerinden `SingleThreadedRandomSource` kurulur. Kaynak sonraki tek `nextDouble()` çekişinden sonra şu ifadeyi kaynak sırasıyla hesaplar:

`coord + (2.0 * random - 1.0) * 1.0E-7 / 2.0`

Seed türetimi nedeniyle aynı koordinat deterministik bir gürültü sonucu üretir; koordinatın pozitif/negatif kesirli olması floor yönünü ve dolayısıyla seed'i değiştirir. `Mth.floor` dönüşümünde Java'nın `double` → `int` daraltma sınırları korunur. F64 çarpma, çıkarma, çarpma, bölme ve toplama sırası değiştirilemez; signed zero, subnormal, Infinity ve NaN sonuçları raw bitlerle gözlenir.

Oracle, Java `Mth.wobble(double)` çağrısını doğrudan yapar ve sonucu `result.bits` altında raw f64 bit string olarak döndürür. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:670-672`, `net/minecraft/util/Mth.java:66-68`, `net/minecraft/util/RandomSource.java:31-32`, `net/minecraft/world/level/levelgen/SingleThreadedRandomSource.java:15-44` ve `net/minecraft/world/level/levelgen/BitRandomSource.java:47-54`.

Kapsam dışı kalanlar: no-arg `createThreadLocalInstance()`, diğer `RandomSource` implementasyonları, `Mth` random helper overload'ları ve `wobble` sonucunu kullanan istemci çağrı zincirleri.
