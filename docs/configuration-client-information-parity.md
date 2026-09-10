# P07 — ClientInformation ve configuration unit codec doğrulaması

Java oracle, `ServerboundClientInformationPacket.STREAM_CODEC` ile üç configuration unit codec'ini gerçek FriendlyByteBuf/ByteBuf üzerinde doğrudan çağırır. Rust adapter alan değerlerini, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 38 özgün senaryo kapsandı: üç unit packet için toplam 15 vaka; ClientInformation için 6 yazma ve 17 okuma vakası. Dil sınırları, Unicode, signed/unsigned byte dönüşümleri, geçerli ve geçersiz enum ordinal'leri, noncanonical boolean byte'ları, kesik alanlar, trailing veri ve prefix doğrulandı.

Wire ailesinin 316 senaryolu odaklı capture kimliği `02697dea4321afbf79133771da7074f3666337febc32c494b3e4677abd495d6b`; `verify-xrj1e2ic`, debug `compare-vg5_vz8r` ve release `compare-y_jgtc9d` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 38 vaka 38/38 eşleşti.

Bu dilimden sonraki birleşik corpus 3612 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=316`. Birleşik capture kimliği `bc194959d690dba988db1ec87157b8c8adb714a12043f2bdd8ea4106b383acb2`; debug `compare-zspio97` ve release `compare-zegngtcb` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç configuration packet payload sınırını doğrular. Configuration listener/state transition, registry data ve gerçek play phase bağlantısı sonraki P07/P08 dilimlerindedir.
