# Mth açı sarma ve fark davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `wrapDegrees90(float)`, `degreesDifference(float, float)` ve `degreesDifferenceAbs(float, float)` yollarını kapsar.

`wrapDegrees90`, açıyı Java f32 remainder işlemiyle `90.0F` değerine göre sarar. `45.0F` ve üstü sonuçtan `90.0F` çıkarılır; `-45.0F` altı sonuca `90.0F` eklenir. Böylece sınırların karşılaştırma yönü korunarak sonuç `[-45, 45)` aralığına taşınır. Remainder sırası, signed zero ve özel değer sonuçları sözleşmenin parçasıdır.

`degreesDifference`, önce `toAngle - fromAngle` f32 çıkarımını yapar ve sonucu `wrapDegrees(float)` ile `[-180, 180)` aralığına taşır. `degreesDifferenceAbs`, bu sonuca Java `abs(float)` uygular; negatif sıfırın pozitif sıfıra ve NaN payload bitlerinin Java `Math.abs` kuralına göre davranması korunur.

Senaryo girdileri raw f32 bit string'leridir. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu raw f32 bit string'i olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:242-261`.

Kapsam dışı kalanlar: `rotateIfNecessary`, `approach`, `approachDegrees`, diğer `wrapDegrees` overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri.
