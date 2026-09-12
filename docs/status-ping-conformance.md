# P06.a — Status request ve ping/pong sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundStatusRequestPacket.STREAM_CODEC` `StreamCodec.unit(INSTANCE)` olduğu için yazmada hiçbir bayt üretmez; okumada input'u incelemeden aynı singleton'ı döndürür. Prefix ve trailing baytlar codec'e ait değildir ve korunur.

`ServerboundPingRequestPacket` ile `ClientboundPongResponsePacket` bir signed Java `long` taşır. Değer sekiz bayt big-endian olarak yazılır ve okunur. Rust API'si `i64` kullanır; sekiz bayttan kısa input `BufferUnderflow` üretir ve atomic `readLong` davranışıyla input'u ilerletmez.

Bu dilim packet payload codec'leriyle sınırlıdır. VarInt21 frame sınırı, status JSON cevabının `ServerStatus` codec'i ve tam protocol state transition ayrı dilimlerdir. Ham status response framing'i ile socket üzerindeki ping echo'su [ayrı bağlantı sözleşmesinde](status-connection-conformance.md) ölçülür.


## Client-parity doğrulaması

Java oracle, status request ile ping/pong `STREAM_CODEC` yollarını gerçek Netty buffer'ları üzerinde doğrudan çağırır. Rust adapter singleton sonucu, yazılan baytları, signed long değerini, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 25 özgün senaryo kapsandı: status request için 2 yazma ve 3 okuma; ping/pong için 6 yazma ve her yönde 7 okuma vakası. Unit codec'in prefix/trailing tüketimsizliği, sıfır/negatif/minimum/maksimum long, sabit byte deseni, prefix, trailing ve yedi baytlık underflow denetlendi.

Wire ailesinin 250 senaryolu odaklı capture kimliği `ffac2b8a79961c5ea39fe9508b567373e5c64948a9653c21d5fbc0c3f8266a69`; `verify-20_5ug5i`, debug `compare-lmfvlh1n` ve release `compare-jagqnu42` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 25 vaka 25/25 eşleşti.

Bu packet’ler eklendikten sonraki birleşik corpus 3546 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=250`. Birleşik capture kimliği `3298d7861ee33b75319509c05b7e12dc218b6c43b34fc7357ea7a79ed2bc4e1b`; debug `compare-tuwq8bxc` ve release `compare-ebt1eln` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç payload codec sınırını doğrular. Status JSON serileştirmesi ve tam protocol state geçişi ayrı kapsamdır; ham status response framing'i ve socket ping echo'su [ayrı vanilla bağlantı kaydında](status-connection-conformance.md) doğrulanmıştır.
