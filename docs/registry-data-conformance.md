# P07.b — Registry data packet sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundRegistryDataPacket` önce registry Identifier'ını, sonra entry listesinin VarInt count'unu taşır. Her `PackedRegistryEntry` entry Identifier'ını, bir optional boolean'ı ve boolean true ise genel NBT `TAG` payload'ını izler.

Optional false yalnızca `00` byte'ıdır ve ardından NBT marker gelmez. Optional true `01` byte'ından sonra NBT type byte'ı ve type'a ait payload gelir; root name taşınmaz. `TAG` EndTag'i non-null codec'e kabul etmez. Optional boolean Java gibi sıfır dışı byte'ları true sayar. Entry listesi sıralıdır; duplicate entry Identifier'ları da listede ayrı ayrı kalır.

Identifier ve liste count hataları packet'in o noktadaki Java tüketim sırasını izler. Negatif list count Java `ArrayList` kapasite yolunda decoder hatasına, fazla uzun VarInt `too_big` sonucuna gider. Optional byte eksikse `buffer_underflow`; true sonrasında EndTag `decoder_error`; NBT okuma IO/eksik gövde yolu `encoder_error` olarak sınıflandırılır.

Rust tarafı entry verisini mevcut `FieldValue` NBT modelinin herhangi bir scalar, array, list veya compound değeri olarak taşır. `read_any_nbt`/`write_any_nbt` root marker ve payload'ı işler; compound-only eski NBT API'si aynı ortak yolu kullanır. Bu dilim format, liste/optional tüketimi ve küçük sentetik NBT değerleriyle sınırlıdır; NBT quota/depth politikası ve registry entry'lerinin client registry state'ine uygulanması ayrı kapsamdır.


## Client-parity doğrulaması

Java oracle, `ClientboundRegistryDataPacket.STREAM_CODEC`, `RegistrySynchronization.PackedRegistryEntry.STREAM_CODEC` ve iç `ByteBufCodecs.TAG` yolunu gerçek FriendlyByteBuf üzerinde doğrudan çağırır. Rust adapter registry Identifier'ını, entry listesini, optional NBT tanımını, yazılan baytları, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

Bu dilimde 27 özgün senaryo kapsandı: yedi encode ve 20 decode vakası. Boş/çoklu entry, absent/scalar/list/compound tag, noncanonical optional boolean, trailing veri, registry/entry Identifier ve list count eksikleri, optional/NBT eksikleri, EndTag reddi ve fazla uzun count VarInt doğrulandı.

Wire ailesinin 434 senaryolu odaklı capture kimliği `54606c264a204b48400905497393d0110e38a1db3a54da5dd32b0c1d192298ab`; `verify-vc49w4ok`, debug `compare-1m060wla` ve release `compare-cewrquoh` raporlarının tamamı fark üretmedi. Birleşik 3730 senaryolu capture kimliği `2f8f115aa25ba1b2a30222383d5c6599e02c43bdccfe7f4219fb222a7240bf1d`; debug `compare-zddpkqwv` ve release `compare-hswwi5gg` raporlarının tamamı geçti.

Bu sonuç registry-data payload eşitliğini doğrular. NBT quota/depth sınırları, registry entry'lerinin gerçek registry'lere çözülmesi, `RegistryDataCollector` birikimi ve configuration→play state geçişi sonraki P03.c/P07.c entegrasyon kapsamındadır.
