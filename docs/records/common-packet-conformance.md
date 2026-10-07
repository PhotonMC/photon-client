# P06.d — Ortak ping/pong ve keep-alive paket sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundPingPacket` ile `ServerboundPongPacket` bir signed 32-bit integer taşır. `ClientboundKeepAlivePacket` ile `ServerboundKeepAlivePacket` bir signed 64-bit integer taşır. Her dört packet alanını FriendlyByteBuf üzerinde Java primitive `writeInt`/`readInt` veya `writeLong`/`readLong` ile big-endian sırada kodlar.

Yazma yalnızca ilgili sabit genişlikteki baytları ekler ve mevcut prefix'i korur. Okuma alan genişliğinden kısa buffer'da underflow üretir ve primitive okuma atomic olduğu için input tüketmez; yeterli buffer'da yalnızca alanı tüketir ve trailing veriyi bırakır. Packet'ler için ek aralık veya değer doğrulaması yoktur.

Rust tarafı dört packet'i ortak public alanı `id` ve `new`/`encode`/`decode` API'siyle taşır. Bu dilim packet payload, byte sırası ve buffer tüketimiyle sınırlıdır; keep-alive yanıt zamanlaması, disconnect/reconnect ve gerçek connection state P06.d entegrasyon kapsamıdır.


## Client-parity doğrulaması

Java oracle, dört common packet'in gerçek `STREAM_CODEC` yollarını FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter packet yönünü, signed değeri, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 48 özgün senaryo kapsandı: dört packet için beş yazma ve yedi okuma vakası. Signed int/long sıfır, negatif bir, minimum, maksimum ve bağımsız desen değerleri; yazma prefix'i, okuma trailing verisi ve alan genişliğinden bir bayt kısa underflow durumları doğrulandı.

Wire ailesinin 364 senaryolu odaklı capture kimliği `b287cfd807713506b20bde46e0395eafd5dd8793c2d8cc70ceae762ced18232c`; `verify-6m3_t27i`, debug `compare-2xadwmgl` ve release `compare-pfhb2wbs` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 48 vaka 48/48 eşleşti.

Bu dilimden sonraki birleşik corpus 3660 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=364`. Birleşik capture kimliği `342b819c9c8ebbe8f83f134a84cd81ac09d65394f47c12cb9dec5b1d1cc7a0b2`; debug `compare-qma4il3h` ve release `compare-3z7rjdv5` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç common packet payload eşitliğini doğrular. Keep-alive zamanlayıcısı, yanıt sırası, disconnect/reconnect ve gerçek socket/protocol state sonraki P06.d entegrasyon testlerindedir.
