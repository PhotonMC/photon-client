# P07.b — Registry data packet codec doğrulaması

Java oracle, `ClientboundRegistryDataPacket.STREAM_CODEC`, `RegistrySynchronization.PackedRegistryEntry.STREAM_CODEC` ve iç `ByteBufCodecs.TAG` yolunu gerçek FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter registry Identifier'ını, entry listesini, optional NBT tanımını, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 27 özgün senaryo kapsandı: yedi encode ve 20 decode vakası. Boş/çoklu entry, absent/scalar/list/compound tag, noncanonical optional boolean, trailing veri, registry/entry Identifier ve list count eksikleri, optional/NBT eksikleri, EndTag reddi ve fazla uzun count VarInt doğrulandı.

Wire ailesinin 434 senaryolu odaklı capture kimliği `54606c264a204b48400905497393d0110e38a1db3a54da5dd32b0c1d192298ab`; `verify-vc49w4ok`, debug `compare-1m060wla` ve release `compare-cewrquoh` raporlarının tamamı fark üretmedi. Birleşik 3730 senaryolu capture kimliği `2f8f115aa25ba1b2a30222383d5c6599e02c43bdccfe7f4219fb222a7240bf1d`; debug `compare-zddpkqwv` ve release `compare-hswwi5gg` raporlarının tamamı geçti.

Bu sonuç registry-data payload eşitliğini doğrular. NBT quota/depth sınırları, registry entry'lerinin gerçek registry'lere çözülmesi, `RegistryDataCollector` birikimi ve configuration→play state geçişi sonraki P03.c/P07.c entegrasyon kapsamındadır.
