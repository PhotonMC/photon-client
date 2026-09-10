# P06.b — Login hello codec doğrulaması

Java oracle, 26.2 istemcisindeki `ServerboundHelloPacket.STREAM_CODEC`'i gerçek `FriendlyByteBuf` üzerinde doğrudan çağırır. Rust adapter isim UTF-16 snapshot'ını, UUID long'larını, encode çıktısını, tüketimi, kalan byte'ları ve hata kategorisini karşılaştırır.

14 özgün login hello senaryosu kapsandı: 5 yazma ve 9 okuma vakası. Boş/Unicode/16 birim isim, yazma sınırı, prefix, trailing veri, kesik isim, encoded/decoded isim hatası, UUID'nin ilk ve ikinci yarı underflow'u ve fazla uzun isim VarInt'i denetlendi.

Java 25.0.4.1 / Linux x86_64 üzerinde odaklı capture kimliği `fb8431b38d799e759a2ad749056c418ba959cc686b992304f3033535600002b9`; `verify-tg36fg_6`, debug `compare-bpzqojx1` ve release `compare-5jpy072u` raporları fark üretmedi. Login hello alt kümesi 14/14 eşleşti.

Birleşik doğrulama corpus'u 3490 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=194`. Capture kimliği `3f9c8b6d0c4eb140d51c97e347bac0115ca5a68e601a86c97a73622bc6d8b8e5`; `verify-zu15xkpg`, debug `compare-5z6q2aso` ve release `compare-6tjk1hjz` raporlarının tamamı geçti.

Bu sonuç P06.b'nin packet codec sınırını doğrular. Login challenge/key exchange, compression/encryption, state transition ve Pumpkin ile gerçek bağlantı akışı sonraki dilimlerdedir.
