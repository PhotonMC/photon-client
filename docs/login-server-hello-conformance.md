# P06.b — Clientbound login hello codec sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, login sunucusunun client'a gönderdiği `ClientboundHelloPacket` alanlarını kapsar. Alanlar server id UTF-8 metni, public key byte array'i, challenge byte array'i ve authenticate boolean'ı sırasıyla taşır. Server id yazılırken genel 32767 UTF-16 birim sınırı, okunurken 20 birim sınırı kullanılır.

Byte array alanlarının her biri VarInt uzunluk ve ham byte gövdesinden oluşur. Java readable byte sayısını length prefix okunmadan önce sınır olarak alır; bu nedenle prefix toplamı yetmiyorsa decoder hatası, prefix okunduktan sonra `readBytes` gövdeyi tamamlayamıyorsa buffer underflow gözlenir. Boolean alanında sıfır byte false, diğer her byte true'dur.

Rust `ClientboundHelloPacket::encode` ve `decode`, server id'nin UTF-16 unit gösterimini, byte array sırasını, UUID dışı ham byte'ları, kısmi tüketimi ve boolean kuralını taşır. Bu sözleşme public key'in kriptografik olarak doğrulanmasını, encryption cipher'larını, authentication servisini, compression handler'ını ve login state geçişini kapsamaz.

Test girdileri özgün sentetik server id birimleri, byte dizileri ve wire verileridir. Java 26.2 referans kaynakları ve capture çıktıları `.data/` altında yerel tutulur.


## Client-parity doğrulaması

Java oracle, 26.2 istemcisindeki `ClientboundHelloPacket.STREAM_CODEC`'i gerçek `FriendlyByteBuf` üzerinde doğrudan çağırır. Rust adapter server id UTF-16 snapshot'ını, public key/challenge byte'larını, boolean'ı, encode çıktısını, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

17 özgün senaryo kapsandı: 5 yazma ve 12 okuma vakası. Boş/Unicode/20 ve 21 birim server id, prefix, trailing veri, encoded/decoded id sınırı, byte array body eksikliği, boolean underflow/noncanonical değer, negatif array size ve fazla uzun VarInt denetlendi.

Java 25.0.4.1 / Linux x86_64 üzerinde odaklı capture kimliği `ff841d76d465a8632668d876a26bd38a7fb8497519e30af767ce2f5694cc8143`; `verify-7hhai7ml`, debug `compare-_tj43x7u` ve release `compare-bgnjd63b` raporları fark üretmedi. Clientbound hello alt kümesi 17/17 eşleşti.

Bu packet eklendikten sonraki birleşik corpus 3521 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=225`. Birleşik capture kimliği `8caa126e3ab721369267d91d226301863bef39b68d74205175a9f3ce2ca1a0c4`; `verify-ai_dqm2s`, debug `compare-62syulpe` ve release `compare-iwk2cson` raporlarının tamamı geçti.

Bu sonuç packet codec sınırını doğrular. Public-key parse/authentication, encryption, compression pipeline, login state transition ve vanilla 26.2 ile gerçek bağlantı sonraki dilimlerdedir.
