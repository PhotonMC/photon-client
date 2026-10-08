# Mth güç-of-two yardımcıları davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `smallestEncompassingPowerOfTwo(int)`, `smallestSquareSide(int)`, `isPowerOfTwo(int)`, `isPowerOfTwo(long)`, `ceillog2(int)` ve `log2(int)` yordamlarını kapsar. Her yordam için sınır ve bit deseni odaklı 10 sentetik senaryo hazırlanır.

`smallestEncompassingPowerOfTwo`, `input - 1`, işaretsiz sağa değil Java signed sağa kaydırma, OR zinciri ve `+1` adımlarını i32 taşmasıyla yürütür. Bu nedenle negatif ve yüksek bit girdileri genel bir pozitif sayı varsayımıyla dönüştürülemez. `smallestSquareSide`, negatif item count'u reddeder; geçerli girdilerde `Math.sqrt` ve `ceil` sonucunu int'e daraltır.

`isPowerOfTwo` her genişlikte sıfırı reddeder ve `input & (input - 1)` bit testini kullanır. `ceillog2`, güç-of-two olmayan girdiyi önce `smallestEncompassingPowerOfTwo` ile dönüştürür, sonra Java'nın sabit De Bruijn tablosundan indeks alır. `log2`, aynı sonucu güç-of-two olmayan girdilerde bir azaltır; sıfır ve yüksek bit sınırları da bu kaynak yoluyla gözlenir.

Behavior runner bütün tamsayı girdilerini signed decimal string olarak taşır. Tamsayı sonuçları da signed decimal string, `isPowerOfTwo` sonuçları ise JSON boolean olarak döner. Java oracle `Mth` yordamlarını doğrudan çağırır; beklenen sonuçlar Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:279-315`.

Kapsam dışı kalanlar: `getInt`, `frac`, `getSeed`, rastgele UUID üretimi, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları, renk dönüşümü ve `binarySearch`.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `smallest_encompassing_power_of_two`, `smallest_square_side`, `is_power_of_two_i32`, `is_power_of_two_i64`, `ceillog2` ve `log2` yordamları, Java `Mth` güç-of-two yardımcılarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; tamsayı sonuçları ve boolean sonuçları Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Altı yordam için 10'ar olmak üzere 60 özgün senaryo hazırlandı. i32 taşması, negatif/yüksek bit girdileri, zero/non-power ayrımı, De Bruijn indekslemesi ve square-side sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2299 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2299/2299 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2801/2801 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth güç-of-two yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `frac`, `getSeed`, `atan2`, ters karekök, renk dönüşümü ve `binarySearch` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
