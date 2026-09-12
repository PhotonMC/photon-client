# P08.b/c — Chunk ve ışık paketinin yapısal sözleşmesi

Bu dilim, Java Edition 26.2'nin `ClientboundLevelChunkWithLightPacket` gövdesini ilk dünya state'ine dönüştürmeden tüketir. Amaç, gerçek vanilla sunucusundan gelen chunk koordinatını, chunk veri bölümlerini ve ışık listelerini doğru sınırlarla ayırmak; eksik veya trailing byte içeren gövdeyi başarı saymamaktır.

## Sabit referans

- Java Edition 26.2
- Protokol 776
- Java 25
- Resmi vanilla 26.2 dedicated server, offline yerel koşu

Java kaynak incelemesinde `ClientboundLevelChunkWithLightPacket` içindeki x/z ve alt codec sırası, `ClientboundLevelChunkPacketData` içindeki heightmap, section buffer ve block entity alanları, `ClientboundLightUpdatePacketData` içindeki dört bitset ve iki ışık listesi kullanıldı. `FriendlyByteBuf.readLongArray`, `readNbt` ve byte-array sınırları da kontrol edildi. Orijinal kaynak depoya kopyalanmadı.

## Java wire sırası

Bir clientbound chunk packet'i aşağıdaki alanları taşır:

1. Chunk `x` ve `z`: big-endian signed i32.
2. Heightmap map'i: VarInt kayıt sayısı; her kayıtta heightmap türü VarInt ve bir length-prefixed long array.
3. Section buffer: VarInt byte uzunluğu ve en fazla `2_097_152` byte. Buffer içinde her section; non-empty block ve fluid signed short'ları, block-state palet konteyneri ve biome palet konteyneri taşır. Her konteyner bits-per-entry byte'ı, gerektiğinde palette ID VarInt'leri ve Java `SimpleBitStorage` düzenindeki sabit long dizisini kullanır.
4. Block entity listesi: VarInt kayıt sayısı; her kayıtta packed x/z byte'ı, y signed short'u, block entity registry VarInt'i ve nullable NBT. NBT kökünde `TAG_End` null anlamına gelir; dolu değer Java'daki `CompoundTag` olmalıdır.
5. Dört ışık maskesi: sky, block, empty sky ve empty block için birer length-prefixed long array.
6. Sky ve block update listeleri: Her biri VarInt kayıt sayısı ve her kayıt için en fazla 2048 byte'lık length-prefixed byte array.

`src/connection.rs` içindeki `parse_chunk_packet`, bu sıranın tamamını tüketir. Heightmap kayıtlarının type ID'si ve long sayısı, dört maskenin long sayısı, iki update listesindeki her byte array'in uzunluğu ve chunk alt alanlarının sınırları `ChunkPacketObservation` içinde gözlenebilir kılınır. Section buffer'ı `parse_section_data` ile byte sonuna kadar tüketir; her section'ın iki sayacını, block-state/biome bits-per-entry değerini, yerel palette giriş sayısını, palette global ID'lerini ve sabit storage long sayısını `ChunkSectionObservation` içinde kaydeder. Storage'ın ilk hücre değerleri ve distinct global ID sayısı da Java'nın düşük bitten başlayan hücre sırasıyla gözlenir. Block state için Java 1–4 bit girişleri 4 bit linear palette'e, 5–8 bit girişleri hashmap palette'e, daha geniş değerleri global palette'e yönlendirir; biome için linear eşik 3 bittir. Storage uzunluğu Java'nın `64 / bits` tam sayı bölümüyle hesaplanır. Payload sonunda byte kalırsa `chunk trailing payload` hatası verir; sınır, negatif count, kesik alan ve geçersiz block entity NBT de başarı sayılmaz.

## Rust sınırı

Bu sözleşmede heightmap type ID'leri veya block entity registry ID'leri çözümlenmez. Section palette ID'leri ve storage'daki bütün block/biome hücreleri global ID'ye çözülür; registry nesnesi ve block entity state eşlemesi yapılmaz. Işık update byte'larının uzunluğu kaydedilir, nibble değerleri ve ışık maskesi semantiği çözülmez. İlk client world cache uygulaması ve section sorguları [P08.a/b kayıt belgesinde](client-world-state-conformance.md) tutulur; dimension state, chunk load/update/unload yaşam döngüsü ve entity bağlantısı sonraki P08 dilimleridir.

Yerel unit testleri Java packet sırasını boş gövde, nullable block entity NBT ve trailing byte senaryolarıyla kontrol eder. Canlı kanıt [vanilla chunk parity kaydındadır](chunk-packet-parity.md).
