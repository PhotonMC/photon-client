# P07.a — Enabled features packet codec doğrulaması

Java oracle, `ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC`'i gerçek FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter feature listesini, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 23 özgün senaryo kapsandı: yedi encode ve 16 decode vakası. Boş/default namespace ve canonical identifier'lar, birden fazla feature'ın Java HashSet sırası, duplicate tekilleştirme, encode UTF-8 sınırı, trailing veri, string prefix/body underflow, invalid namespace/path, malformed UTF-8, encoded/decoded uzunluk sınırları, negatif count ve fazla uzun count VarInt doğrulandı.

Wire ailesinin 387 senaryolu odaklı capture kimliği `007b6f762907489d48475d8d4928786a9e22493a43e7f823629c3d248a4bdacf`; `verify-hy7uaxxq`, debug `compare-pbvara1r` ve release `compare-906xa6sx` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 23 vaka 23/23 eşleşti.

Bu dilimden sonraki birleşik corpus 3683 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=387`. Birleşik capture kimliği `5034630c96aee12bba3d6e03a2c7a90157dc62e9be2b92911a16e7f03016887a`; debug `compare-u0lwjoz0` ve release `compare-s3c1thj0` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç enabled-features packet payload eşitliğini doğrular. Feature registry çözümü, tag/registry biriktirme ve configuration→play geçişi sonraki P07.b/c entegrasyon testlerindedir.
