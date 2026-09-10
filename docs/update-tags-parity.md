# P07.b — Update tags packet codec doğrulaması

Java oracle, `ClientboundUpdateTagsPacket.STREAM_CODEC` ve iç `TagNetworkSerialization.NetworkPayload` yollarını gerçek FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter registry/tag listelerini, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 20 özgün senaryo kapsandı: beş encode ve 15 decode vakası. Boş ve boş registry payload'ı, birden fazla tag'ın HashMap sırası, duplicate tag'ın son değeri, signed/çok baytlı id VarInt'leri, trailing veri, registry/tag/value count underflow'ları, negatif value count'un boş liste davranışı, invalid identifier ve fazla uzun count VarInt doğrulandı. Java'nın identity hash'e bağlı çoklu üst registry map sırası bu vaka kümesine alınmadı.

Wire ailesinin 407 senaryolu odaklı capture kimliği `8c2f3e4335e4e7709dce15cb105c224f10a7784edace7e1f04472891117c00bb`; `verify-45o1gfnr`, debug `compare-qdxlavsp` ve release `compare-im09iuk_` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 20 vaka 20/20 eşleşti.

Bu dilimden sonraki birleşik corpus 3703 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=407`. Birleşik capture kimliği `87d867c917ad575a3e3bfee10deb09cd5241dd84d4993a7b8de6177730c8e4f3`; debug `compare-5e7pcauo` ve release `compare-fc7xrf9_` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç update-tags payload eşitliğini doğrular. Tag'lerin registry'ye uygulanması, çoklu registry toplama ve configuration listener state geçişi sonraki P07.b/c entegrasyon testlerindedir.
