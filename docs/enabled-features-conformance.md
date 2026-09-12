# P07.a — Enabled features packet sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundUpdateEnabledFeaturesPacket` payload'ı önce feature sayısını VarInt, sonra her feature'ı `FriendlyByteBuf` üzerindeki varsayılan 32767 UTF-16 birimlik UTF-8 alanı olarak taşır.

Her feature `Identifier.parse` ile çözülür. İlk `:` namespace/path ayracıdır; ayracın başında veya hiç bulunmamasında namespace `minecraft` olur. Namespace yalnızca küçük ASCII harf, rakam, `_`, `-`, `.` içerir ve `..` olamaz; path aynı karakterlere ek olarak `/` kabul eder. Boş path Java tarafından geçerlidir. Geçersiz identifier decode sırasında decoder hatasıdır.

Packet alanı Java `Set<Identifier>` olduğu için duplicate feature'lar tekilleşir. Java encode yolu varsayılan `HashSet` iterasyon sırasını, decode yolu ise count ile oluşturulan `HashSet` sırasını kullanır; bucket kapasitesi bu iki yolu ayırabilir. Sabit alan genişliğinden kısa string/body okumaları Java UTF-8 codec'inin tüketim kuralını korur: uzunluk prefix'i okunur, gövde eksikse gövde tüketilmeden decoder hatası verilir. Count VarInt'i tüketilir; negatif count decoder hatasıdır.

Rust tarafı Identifier doğrulamasını, Java String hash/spread hash tabanlı HashSet sırasını, duplicate tekilleştirmeyi ve collection'ın kısmi okuma/yazma davranışını taşır. Bu dilim yalnızca feature payload'ı kapsar; `FeatureFlagRegistry.fromNames`, registry biriktirme ve configuration listener state geçişi ayrı kapsamdır.


## Client-parity doğrulaması

Java oracle, `ClientboundUpdateEnabledFeaturesPacket.STREAM_CODEC`'i gerçek FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter feature listesini, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 23 özgün senaryo kapsandı: yedi encode ve 16 decode vakası. Boş/default namespace ve canonical identifier'lar, birden fazla feature'ın Java HashSet sırası, duplicate tekilleştirme, encode UTF-8 sınırı, trailing veri, string prefix/body underflow, invalid namespace/path, malformed UTF-8, encoded/decoded uzunluk sınırları, negatif count ve fazla uzun count VarInt doğrulandı.

Wire ailesinin 387 senaryolu odaklı capture kimliği `007b6f762907489d48475d8d4928786a9e22493a43e7f823629c3d248a4bdacf`; `verify-hy7uaxxq`, debug `compare-pbvara1r` ve release `compare-906xa6sx` raporlarının tamamı fark üretmedi. Bu dilimin eklediği 23 vaka 23/23 eşleşti.

Bu dilimden sonraki birleşik corpus 3683 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=387`. Birleşik capture kimliği `5034630c96aee12bba3d6e03a2c7a90157dc62e9be2b92911a16e7f03016887a`; debug `compare-u0lwjoz0` ve release `compare-s3c1thj0` raporları geçti. Birleşik capture’ın Java referansı iki tekrar çalıştırmasıyla kararlı bulundu.

Bu sonuç enabled-features packet payload eşitliğini doğrular. Feature registry çözümü, tag/registry biriktirme ve configuration→play geçişi sonraki P07.b/c entegrasyon testlerindedir.
