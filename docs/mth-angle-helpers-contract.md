# Mth açı yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `packDegrees(float)`, `unpackDegrees(byte)` ve `wrapDegrees(long)` yollarını kapsar.

`packDegrees`, f32 açıyı Java f32 çarpma/bölme sırasıyla 256/360 ölçeğine taşır, `floor` uygular ve sonucu byte genişliğine daraltır. NaN, sonsuzluk ve 256 katı sınırlarında Java'nın float-to-int ve int-to-byte dönüşümleri korunur. `unpackDegrees`, signed byte değeri önce int çarpımına, ardından f32 bölmeye çevirir.

`wrapDegrees(long)`, long açıdan Java signed `% 360L` kalanını alır ve sonucu f32'ye daraltır. `180` ve üstü 360 azaltılır, `-180` altı 360 artırılır; sonuç `[-180, 180)` aralığındadır. Long sınırlarında kalan ve son f32 bitleri gözlenebilir sözleşmenin parçasıdır.

Senaryo girdileri pack için raw f32 bitleri, unpack için signed byte decimal string'i, long wrap için signed long decimal string'i taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu byte decimal string'i veya raw f32 bitleri olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:178-188` (`packDegrees`, `unpackDegrees`), `:203-214` (`wrapDegrees(long)`).

Kapsam dışı kalanlar: `wrapDegrees(int)` ve float/double overload'ları, `wrapDegrees90`, derece farkı/rotation yardımcıları, sıfıra bölme hata biçimi ve diğer Mth açı/tamsayı yardımcıları.
