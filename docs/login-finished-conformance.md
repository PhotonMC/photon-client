# P06.b — Login completion packet sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundLoginAcknowledgedPacket.STREAM_CODEC` status request ile aynı unit codec davranışına sahiptir: encode hiçbir bayt yazmaz, decode input'u tüketmeden singleton döndürür.

`ClientboundLoginFinishedPacket` önce GameProfile UUID'sini, 16 UTF-16 birimlik adı ve en fazla 16 property kaydını, sonra session UUID'sini taşır. Her property adı 64, değeri 32767, nullable signature değeri 1024 UTF-16 birimiyle `Utf8String` sınırlarına tabidir. Property kaydı name, value ve signature-present boolean sırasındadır; signature varsa boolean'dan sonra string gelir.

Rust tarafı GameProfile alanlarını UTF-16 birimleriyle, UUID'leri `binary::Uuid` ile ve property kayıtlarını giriş sırasındaki `Vec<ProfileProperty>` ile taşır. Count sınırı count VarInt'inden sonra, UUID ve önceki alanlar tüketilmişken uygulanır. Kısa UUID/alan okumaları Java'nın alan sırasını ve kısmi tüketimini korur.

Bu dilim packet payload codec'leriyle sınırlıdır. Authentication, encryption, login listener/state transition ve configuration phase packet'leri sonraki dilimlerdir.


## Client-parity doğrulaması

Java oracle, login acknowledgement ve `ClientboundLoginFinishedPacket.STREAM_CODEC` yollarını gerçek Netty buffer'ları ve authlib `GameProfile` nesneleri üzerinde doğrudan çağırır. Rust adapter payload baytlarını, decoded UUID/ad/property snapshot'ını, session UUID'sini, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 28 özgün senaryo kapsandı: acknowledgement için 2 yazma ve 3 okuma; login finished için 9 yazma ve 14 okuma vakası. Unit ack tüketimsizliği, boş/Unicode/sınır ad, property count ve string sınırları, imzalı/imzasız property, negatif count, profile/session UUID underflow, property alan underflow ve trailing veri denetlendi.

Wire ailesinin 278 senaryolu odaklı capture kimliği `574145467c9ddb4cae4b98934f23f6881be7cfd14135d05a74c9c5612bf32a89`; `verify-270ei0iq`, debug `compare-uqe0gtnk` ve release `compare-nl98ndh6` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 28 vaka 28/28 eşleşti.

Bu dilimden sonraki birleşik corpus 3574 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=278`. Birleşik capture kimliği `1c7541b79830a882b8c5d237bbdb68dd0813fef5c6142e1379af340cd86fd4a8`; debug `compare-ft39g1_7` ve release `compare-6mzorbny` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç login completion payload sınırını doğrular. Authentication, encryption, listener/state transition ve configuration phase packet akışı sonraki P06/P07 dilimlerindedir.
