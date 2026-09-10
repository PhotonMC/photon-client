# P06.b — Clientbound login hello codec sözleşmesi

Bu dilim, login sunucusunun client'a gönderdiği `ClientboundHelloPacket` alanlarını kapsar. Alanlar server id UTF-8 metni, public key byte array'i, challenge byte array'i ve authenticate boolean'ı sırasıyla taşır. Server id yazılırken genel 32767 UTF-16 birim sınırı, okunurken 20 birim sınırı kullanılır.

Byte array alanlarının her biri VarInt uzunluk ve ham byte gövdesinden oluşur. Java readable byte sayısını length prefix okunmadan önce sınır olarak alır; bu nedenle prefix toplamı yetmiyorsa decoder hatası, prefix okunduktan sonra `readBytes` gövdeyi tamamlayamıyorsa buffer underflow gözlenir. Boolean alanında sıfır byte false, diğer her byte true'dur.

Rust `ClientboundHelloPacket::encode` ve `decode`, server id'nin UTF-16 unit gösterimini, byte array sırasını, UUID dışı ham byte'ları, kısmi tüketimi ve boolean kuralını taşır. Bu sözleşme public key'in kriptografik olarak doğrulanmasını, encryption cipher'larını, authentication servisini, compression handler'ını ve login state geçişini kapsamaz.

Test girdileri özgün sentetik server id birimleri, byte dizileri ve wire verileridir. Java 26.2 referans kaynakları ve capture çıktıları `.data/` altında yerel tutulur.
