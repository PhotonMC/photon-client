# P06.c — Login compression threshold codec doğrulaması

Java oracle, 26.2 istemcisindeki `ClientboundLoginCompressionPacket.STREAM_CODEC`'i `FriendlyByteBuf` üzerinde doğrudan çağırır. Rust adapter threshold değerini, VarInt byte'larını, yazılan byte sayısını, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

14 özgün senaryo kapsandı: 7 yazma ve 7 okuma vakası. Sıfır, tek/iki bayt eşikleri, negatif ve maksimum i32 threshold, prefix, trailing veri, noncanonical sıfır, kesik VarInt ve fazla uzun VarInt denetlendi.

Java 25.0.4.1 / Linux x86_64 üzerinde odaklı capture kimliği `d36a36fe7d43fab5aaabb4b41d2d7b738cb388e09b9c0566183c761f7850ce80`; `verify-hv1p6vq9`, debug `compare-0jjo3hrx` ve release `compare-9pppbgc4` raporları fark üretmedi. Compression threshold alt kümesi 14/14 eşleşti.

Bu packet eklendikten sonraki birleşik corpus 3504 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=208`. Birleşik capture kimliği `fd09e0ac958f9a4cfa8bb7f7f74d634ffdd3c3662e15d6fecfde11fc9f5b5846`; `verify-s7ldy3lp`, debug `compare-3fpmctxj` ve release `compare-tqn2f4hv` raporlarının tamamı geçti.

Bu sonuç yalnızca threshold packet codec'ini doğrular. Sıkıştırılmış frame dönüşümü, pipeline handler'ı, login state transition ve Pumpkin ile gerçek bağlantı sonraki P06.c/P06.d dilimlerindedir.
