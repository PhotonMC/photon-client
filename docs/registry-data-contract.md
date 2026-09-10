# P07.b — Registry data packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ClientboundRegistryDataPacket` önce registry Identifier'ını, sonra entry listesinin VarInt count'unu taşır. Her `PackedRegistryEntry` entry Identifier'ını, bir optional boolean'ı ve boolean true ise genel NBT `TAG` payload'ını izler.

Optional false yalnızca `00` byte'ıdır ve ardından NBT marker gelmez. Optional true `01` byte'ından sonra NBT type byte'ı ve type'a ait payload gelir; root name taşınmaz. `TAG` EndTag'i non-null codec'e kabul etmez. Optional boolean Java gibi sıfır dışı byte'ları true sayar. Entry listesi sıralıdır; duplicate entry Identifier'ları da listede ayrı ayrı kalır.

Identifier ve liste count hataları packet'in o noktadaki Java tüketim sırasını izler. Negatif list count Java `ArrayList` kapasite yolunda decoder hatasına, fazla uzun VarInt `too_big` sonucuna gider. Optional byte eksikse `buffer_underflow`; true sonrasında EndTag `decoder_error`; NBT okuma IO/eksik gövde yolu `encoder_error` olarak sınıflandırılır.

Rust tarafı entry verisini mevcut `FieldValue` NBT modelinin herhangi bir scalar, array, list veya compound değeri olarak taşır. `read_any_nbt`/`write_any_nbt` root marker ve payload'ı işler; compound-only eski NBT API'si aynı ortak yolu kullanır. Bu dilim format, liste/optional tüketimi ve küçük sentetik NBT değerleriyle sınırlıdır; NBT quota/depth politikası ve registry entry'lerinin client registry state'ine uygulanması ayrı kapsamdır.
