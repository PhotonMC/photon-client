# Mth binarySearch davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `binarySearch(int, int, IntPredicate)` yordamını kapsar. Yordam `[from, to)` aralığında, koşulun ilk true verdiği konumu arar; koşul hiç true değilse aralığın üst ucunu üretir.

Her döngüde uzunluk Java i32 çıkarma ile, orta değer `from + half` ile ve false dalındaki `middle + 1`/uzunluk güncellemesi i32 aritmetiğiyle hesaplanır. Bu nedenle signed sınır girdilerindeki taşma da davranış sözleşmesinin parçasıdır. Adapter iki monoton koşul biçimi sunar: `ge` için `value >= threshold`, `gt` için `value > threshold`.

Java oracle `Mth.binarySearch` yordamını doğrudan çağırır; beklenen değer Rust uygulamasından türetilmez. Girdiler ve sonuçlar signed decimal i32 string olarak taşınır.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:524-539`.

Kapsam dışı kalanlar: keyfi predicate bileşimleri, `rayIntersectsAABB`, `atan2`, ters karekök yardımcıları ve renk dönüşümleri.
