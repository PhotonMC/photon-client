# Mth güç-of-two yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `smallestEncompassingPowerOfTwo(int)`, `smallestSquareSide(int)`, `isPowerOfTwo(int)`, `isPowerOfTwo(long)`, `ceillog2(int)` ve `log2(int)` yordamlarını kapsar. Her yordam için sınır ve bit deseni odaklı 10 sentetik senaryo hazırlanır.

`smallestEncompassingPowerOfTwo`, `input - 1`, işaretsiz sağa değil Java signed sağa kaydırma, OR zinciri ve `+1` adımlarını i32 taşmasıyla yürütür. Bu nedenle negatif ve yüksek bit girdileri genel bir pozitif sayı varsayımıyla dönüştürülemez. `smallestSquareSide`, negatif item count'u reddeder; geçerli girdilerde `Math.sqrt` ve `ceil` sonucunu int'e daraltır.

`isPowerOfTwo` her genişlikte sıfırı reddeder ve `input & (input - 1)` bit testini kullanır. `ceillog2`, güç-of-two olmayan girdiyi önce `smallestEncompassingPowerOfTwo` ile dönüştürür, sonra Java'nın sabit De Bruijn tablosundan indeks alır. `log2`, aynı sonucu güç-of-two olmayan girdilerde bir azaltır; sıfır ve yüksek bit sınırları da bu kaynak yoluyla gözlenir.

Behavior runner bütün tamsayı girdilerini signed decimal string olarak taşır. Tamsayı sonuçları da signed decimal string, `isPowerOfTwo` sonuçları ise JSON boolean olarak döner. Java oracle `Mth` yordamlarını doğrudan çağırır; beklenen sonuçlar Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:279-315`.

Kapsam dışı kalanlar: `getInt`, `frac`, `getSeed`, rastgele UUID üretimi, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları, renk dönüşümü ve `binarySearch`.
