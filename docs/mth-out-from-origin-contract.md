# Mth outFromOrigin davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `outFromOrigin(int, int, int)` ve `outFromOrigin(int, int, int, int)` overload'larını kapsar.

Üç argümanlı overload, dört argümanlı yolu `stepSize = 1` ile çağırır. Dört argümanlı yol önce `lowerBound <= upperBound` ve `stepSize >= 1` koşullarını denetler. İlk koşul bozulduğunda üst sınır ve alt sınır değerlerini içeren Java `IllegalArgumentException` metni; ikinci koşul bozulduğunda adım değerini içeren ayrı metin oluşur.

Geçerli girdide origin, iki sınır arasına clamp edilerek stream'in ilk değeri olur. Stream predicate'i mevcut cursor'ın origin etrafındaki iki yöndeki mesafelerden en az birinin aralıkta kalıp kalmadığını denetler. Sonraki cursor, Java'nın kaynak sırasına göre pozitif aday ve negatif aday arasında ilerler; aralıkta yalnızca bir yön kaldığında o yön sürdürülür. Liste sırası bu lazy `IntStream` akışının tüketim sırasıdır.

Mesafe, toplama, çıkarma ve karşılaştırmalar Java i32 aritmetiğiyle gözlenir. Bu dilimin sonlu corpus girdileri stream'in sonlanmasını koruyan dar aralıklardır; büyük taşmalı aralıklar ve pratikte sonlanmayabilecek overflow döngüleri kapsama alınmadı.

Behavior runner girdilerinde `mode: "default"` üç argümanlı overload'u, `mode: "step"` dört argümanlı overload'u seçer. Signed i32 alanları decimal string'dir. Başarılı sonuç `outcome: "value"` ve JSON i32 listesi; doğrulama hatası `outcome: "error"` ve Java hata metni olarak taşınır. Java exception sınıf adı yalnızca tanısaldır ve sonuç karşılaştırmasına girmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:730-760`.

Kapsam dışı kalanlar: overflow ile sonlanmayan veya çok büyük stream'ler, bu helper'ın istemci çağrı zincirleri ve sonraki `Mth` yardımcıları.
