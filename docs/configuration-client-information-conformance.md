# P07 — ClientInformation ve configuration unit packet sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundClientInformationPacket` alanları şu sıradadır: UTF-8 language, byte view distance, chat visibility ordinal VarInt, chat colors boolean, byte model customisation, main hand ordinal VarInt, text filtering boolean, allows listing boolean ve particle status ordinal VarInt.

Language okumada 16 UTF-16 birimi sınırına, yazmada `writeUtf(String)` varsayılanı olan 32767 sınıra tabidir. View distance Java signed byte olarak okunur; model customisation unsigned byte olarak okunur. Yazmada iki değer de Java `writeByte` gibi düşük sekiz biti taşır. Enum okumada ordinal doğrudan dizi indeksidir; geçersiz ordinal decoder hatasıdır. Boolean alanlar sıfır dışı byte'ı true kabul eder.

`ServerboundFinishConfigurationPacket`, `ClientboundFinishConfigurationPacket` ve `ServerboundAcceptCodeOfConductPacket` unit codec kullanır. Yazma sıfır byte ekler; okuma input'u ilerletmeden singleton packet döndürür.

Rust tarafı enum'ları açık ordinal değerleriyle, primitive alanları Java byte genişlikleriyle ve unit packet'leri singleton değerlerle taşır. Bu dilim payload codec ve alan tüketimiyle sınırlıdır; configuration listener/state transition, registry verisi ve play phase ayrı kapsamdır.


## Client-parity doğrulaması

Java oracle, `ServerboundClientInformationPacket.STREAM_CODEC` ile üç configuration unit codec'ini gerçek FriendlyByteBuf/ByteBuf üzerinde doğrudan çağırır. Rust adapter alan değerlerini, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 38 özgün senaryo kapsandı: üç unit packet için toplam 15 vaka; ClientInformation için 6 yazma ve 17 okuma vakası. Dil sınırları, Unicode, signed/unsigned byte dönüşümleri, geçerli ve geçersiz enum ordinal'leri, noncanonical boolean byte'ları, kesik alanlar, trailing veri ve prefix doğrulandı.

Wire ailesinin 316 senaryolu odaklı capture kimliği `02697dea4321afbf79133771da7074f3666337febc32c494b3e4677abd495d6b`; `verify-xrj1e2ic`, debug `compare-vg5_vz8r` ve release `compare-y_jgtc9d` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 38 vaka 38/38 eşleşti.

Bu dilimden sonraki birleşik corpus 3612 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=316`. Birleşik capture kimliği `bc194959d690dba988db1ec87157b8c8adb714a12043f2bdd8ea4106b383acb2`; debug `compare-zspio97` ve release `compare-zegngtcb` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç configuration packet payload sınırını doğrular. Configuration listener/state transition, registry data ve gerçek play phase bağlantısı sonraki P07/P08 dilimlerindedir.
