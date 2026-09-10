# P06.b — Login completion codec doğrulaması

Java oracle, login acknowledgement ve `ClientboundLoginFinishedPacket.STREAM_CODEC` yollarını gerçek Netty buffer'ları ve authlib `GameProfile` nesneleri üzerinde doğrudan çağırır. Rust adapter payload baytlarını, decoded UUID/ad/property snapshot'ını, session UUID'sini, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 28 özgün senaryo kapsandı: acknowledgement için 2 yazma ve 3 okuma; login finished için 9 yazma ve 14 okuma vakası. Unit ack tüketimsizliği, boş/Unicode/sınır ad, property count ve string sınırları, imzalı/imzasız property, negatif count, profile/session UUID underflow, property alan underflow ve trailing veri denetlendi.

Wire ailesinin 278 senaryolu odaklı capture kimliği `574145467c9ddb4cae4b98934f23f6881be7cfd14135d05a74c9c5612bf32a89`; `verify-270ei0iq`, debug `compare-uqe0gtnk` ve release `compare-nl98ndh6` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 28 vaka 28/28 eşleşti.

Bu dilimden sonraki birleşik corpus 3574 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=278`. Birleşik capture kimliği `1c7541b79830a882b8c5d237bbdb68dd0813fef5c6142e1379af340cd86fd4a8`; debug `compare-ft39g1_7` ve release `compare-6mzorbny` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç login completion payload sınırını doğrular. Authentication, encryption, listener/state transition ve configuration phase packet akışı sonraki P06/P07 dilimlerindedir.
