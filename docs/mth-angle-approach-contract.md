# Mth açı yaklaşım davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotateIfNecessary(float, float, float)`, `approach(float, float, float)` ve `approachDegrees(float, float, float)` yollarını kapsar.

`rotateIfNecessary`, hedef ile taban arasındaki `degreesDifference` sonucunu `[-maxAngleDiff, maxAngleDiff]` sınırlarında Java f32 `clamp` ile sınırlar ve bu değeri hedef açıdan çıkarır. Sınırlar ayrıca normalize edilmez; ters sınırlar ve özel f32 değerleri Java karşılaştırma sırasına bağlıdır.

`approach`, artışın Java `abs(float)` sonucunu kullanır. Geçerli değer hedeften küçükse toplama sonucunu `[current, target]` aralığında, aksi durumda çıkarma sonucunu `[target, current]` aralığında clamp eder. Eşitlik, signed zero, taşma, Infinity ve NaN dalları bu sıralamanın parçasıdır.

`approachDegrees`, önce `degreesDifference(current, target)` ile açısal farkı `[-180, 180)` aralığına taşır, ardından `current + difference` hedefiyle `approach` yolunu çağırır. Ara f32 toplaması ve artışın mutlak değer alınması korunur.

Senaryo girdileri raw f32 bit string'leridir. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonuçları raw f32 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:263-277`.

Kapsam dışı kalanlar: `getInt`, `smallestEncompassingPowerOfTwo`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri.
