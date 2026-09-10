# P06.b — Login hello codec sözleşmesi

Bu dilim, handshake intention login yolundan sonra gönderilen `ServerboundHelloPacket` alanlarını kapsar. Packet önce adı 16 UTF-16 birimi sınırıyla uzunluklu UTF-8 olarak, sonra profile UUID'sinin most-significant ve least-significant long alanlarını big-endian sırada taşır.

İsim encode ve decode yollarında aynı 16 birim sınırını kullanır. İsim sınırı veya encoded uzunluk hatası UUID yazımından önce oluşur. Decode alanları sırayla tüketir; isim alanı geçerli olduktan sonra UUID'nin ilk sekiz baytı tüketilip ikinci sekiz baytı eksik kalabilir. UUID için ek semantik doğrulama yapılmaz; ham 128 bit desen korunur.

Rust `ServerboundHelloPacket::encode` ve `decode`, UTF-16 unit tabanlı isim, UUID byte sırası, kısmi yazma ve kısmi tüketme davranışını taşır. Bu sözleşme login challenge/key exchange, compression/encryption, connection state ve gerçek server oturumunu kapsamaz.

Test girdileri özgün sentetik isim birimleri, UUID bit desenleri ve wire byte dizileridir. Java 26.2 referans kaynakları ve capture çıktıları `.data/` altında yerel tutulur.
