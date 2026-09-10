# P06.b — Handshake intention codec sözleşmesi

Bu dilim, istemcinin handshake durumundan status, login veya transfer yolunu seçmek için gönderdiği `ClientIntentionPacket` alanlarını kapsar. Alanlar sırayla protocol version VarInt, host adının UTF-8 uzunluklu gösterimi, iki baytlık big-endian port ve intention id VarInt olarak taşınır. Intention id'leri status için 1, login için 2 ve transfer için 3'tür.

Host adı yazılırken Java varsayılan UTF-8 sınırı olan 32767 UTF-16 birimi kullanılır; okunurken packet'in 255 birim sınırı uygulanır. Port yazımı verilen i32 değerinin düşük 16 bitini taşır ve okuma unsigned-short değeri i32 olarak verir. Decode alanları sırayla tüketir; eksik port iki baytı tamamlamadan hata verir, bilinmeyen intention id decoder hatasıdır ve VarInt taşması `too_big` sonucudur.

Rust `ClientIntentionPacket::encode` ve `decode`, UTF-16 unit tabanlı host gösterimiyle bu alan sırasını, kısmi tüketimi ve kısmi yazmayı taşır. Packet codec'i henüz frame başlığı, connection state, status response, login challenge, compression/encryption veya gerçek socket akışını kapsamaz.

Test girdileri özgün sentetik UTF-16 host birimleri ve wire byte dizileridir. Java 26.2 kaynakları ve çalıştırma çıktıları `.data/` altında yerel tutulur.
