# Mth murmurHash3Mixer davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `murmurHash3Mixer(int)` yordamını kapsar. Yordam tek bir signed i32 girdiyi dört unsigned sağa kaydırma/XOR ve iki i32 çarpma adımıyla karıştırır.

Java `>>>` adımları, girdinin signed olması durumunda üst bitleri sıfırla doldurur. Her çarpma Java i32 genişliğinde wrapping davranışla yürür; son XOR da signed i32 sonucu korur. Çıktı signed decimal i32 string olarak taşınır.

Java oracle `Mth.murmurHash3Mixer` yordamını doğrudan çağırır; beklenen değer Rust uygulamasından türetilmez. Senaryolar sıfır, işaretli sınırlar, bit desenleri, kaydırma bitleri ve çarpma taşmalarını kapsar.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:516-522`.

Kapsam dışı kalanlar: `binarySearch`, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları ve renk dönüşümleri.
