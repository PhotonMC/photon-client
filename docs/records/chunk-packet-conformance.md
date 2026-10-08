# P08.b/c — Chunk ve ışık paketinin yapısal sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

`crates/photon-net/src/connection/protocol.rs` içindeki `parse_chunk_packet`, bu sıranın tamamını tüketir. Heightmap kayıtlarının type ID'si ve long sayısı, dört maskenin long sayısı, iki update listesindeki her byte array'in uzunluğu ve chunk alt alanlarının sınırları `ChunkPacketObservation` içinde gözlenebilir kılınır. Section buffer'ı `parse_section_data` ile byte sonuna kadar tüketir; her section'ın iki sayacını, block-state/biome bits-per-entry değerini, yerel palette giriş sayısını, palette global ID'lerini ve sabit storage long sayısını `ChunkSectionObservation` içinde kaydeder. Storage'ın ilk hücre değerleri ve distinct global ID sayısı da Java'nın düşük bitten başlayan hücre sırasıyla gözlenir. Block state için Java 1–4 bit girişleri 4 bit linear palette'e, 5–8 bit girişleri hashmap palette'e, daha geniş değerleri global palette'e yönlendirir; biome için linear eşik 3 bittir. Storage uzunluğu Java'nın `64 / bits` tam sayı bölümüyle hesaplanır. Payload sonunda byte kalırsa `chunk trailing payload` hatası verir; sınır, negatif count, kesik alan ve geçersiz block entity NBT de başarı sayılmaz.

## Rust sınırı

Bu sözleşmede heightmap type ID'leri veya block entity registry ID'leri çözümlenmez. Section palette ID'leri ve storage'daki bütün block/biome hücreleri global ID'ye çözülür; registry nesnesi ve block entity state eşlemesi yapılmaz. Işık update byte'larının uzunluğu kaydedilir, nibble değerleri ve ışık maskesi semantiği çözülmez. İlk client world cache uygulaması ve section sorguları [P08.a/b kayıt belgesinde](client-world-state-conformance.md) tutulur; dimension state, chunk load/update/unload yaşam döngüsü ve entity bağlantısı sonraki P08 dilimleridir.

Yerel unit testleri Java packet sırasını boş gövde, nullable block entity NBT ve trailing byte senaryolarıyla kontrol eder. Canlı kanıt [Vanilla server-conformance doğrulama bölümündedir](#vanilla-server-conformance-doğrulaması).


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11
**Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Sunucu:** Mojang'ın vanilla 26.2 dedicated server JAR'ı, offline yerel koşu
**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu geçici `/tmp/photon-vanilla-26.2-test` dizininde `127.0.0.1:25566`, `online-mode=false`, seed `776`, view/simulation distance `5`, creative ve peaceful ayarlarıyla başlatıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustLight1
cargo run --offline --example connection_probe -- 127.0.0.1:25566 RustLight2
```

İki bağımsız oturumda parser bütün alınan chunk payload'larını hatasız tüketti:

| Gözlem | RustLight1 | RustLight2 |
| --- | ---: | ---: |
| chunk packet | 23 | 21 |
| decoded chunk | 23 | 21 |
| chunk batch finished | 5 | 5 |
| toplam alınan packet | 462 | 475 |
| ilk chunk section sayısı | 24 | 24 |

`decoded chunk` sayısının `chunk packet` sayısına eşit olması, canlı koşuda her chunk gövdesinin parser'dan geçtiğini gösterir. Her iki koşuda da ilk chunk'ın section buffer'ı 24 section olarak ayrıştırıldı. Toplam packet ve chunk sayısı vanilla sunucunun geçici gönderim zamanlamasına bağlıdır.

İlk chunk gözlemleri ve section başlıkları aşağıdaki gibidir:

```text
RustLight1:
first_chunk=x:1,z:-3,heightmaps:3,heightmap_observations:[HeightmapObservation { type_id: 4, word_count: 37 }, HeightmapObservation { type_id: 1, word_count: 37 }, HeightmapObservation { type_id: 5, word_count: 37 }],section_bytes:22265,block_entities:0,sky_mask_words:1,block_mask_words:1,empty_sky_mask_words:1,empty_block_mask_words:1,sky_updates:[2048, 2048, 2048, 2048],block_updates:[2048, 2048]
first_section=non_empty:4096,fluid:0,block_bits:4,block_palette:6,block_palette_ids:[85, 23452, 30417, 6884, 130, 132],block_words:256,block_first_values:[85, 85, 85, 85],block_distinct:6,biome_bits:0,biome_palette:1,biome_palette_ids:[4],biome_words:0,biome_first_values:[4, 4, 4, 4],biome_distinct:1
first_chunk_section_count=24

RustLight2:
first_chunk=x:1,z:-3,heightmaps:3,heightmap_observations:[HeightmapObservation { type_id: 4, word_count: 37 }, HeightmapObservation { type_id: 1, word_count: 37 }, HeightmapObservation { type_id: 5, word_count: 37 }],section_bytes:22265,block_entities:0,sky_mask_words:1,block_mask_words:1,empty_sky_mask_words:1,empty_block_mask_words:1,sky_updates:[2048, 2048, 2048, 2048],block_updates:[2048, 2048]
first_section=non_empty:4096,fluid:0,block_bits:4,block_palette:6,block_palette_ids:[85, 23452, 30417, 6884, 130, 132],block_words:256,block_first_values:[85, 85, 85, 85],block_distinct:6,biome_bits:0,biome_palette:1,biome_palette_ids:[4],biome_words:0,biome_first_values:[4, 4, 4, 4],biome_distinct:1
first_chunk_section_count=24
```

Bu sonuç x/z signed integer alanlarını, üç heightmap kaydını, section buffer sınırını, boş block entity listesini, dört ışık maskesini ve sky/block update listelerini vanilla wire sırasıyla ayırdığımızı gösterir. İki koşudaki heightmap type ID sırası `[4, 1, 5]`, her heightmap'in 37 long verisi ve her ışık update byte array'inin 2048 byte uzunluğu aynı kaldı. Her iki koşudaki 24 section'ın section sayaçları, palette bits değerleri, palette global ID listeleri ve storage word uzunlukları Java layout'una göre tüketildi; parser payload sonunda trailing byte bırakmadı. İlk chunk'ın konumu ve palette giriş sayısı, sunucunun packet gönderim sırasına göre değişebildiği için iki koşu ayrı kaydedildi.

## Açık kapsam

Bu kayıt packet gövdesinin, section palette storage düzeninin ve heightmap/light byte sınırlarının yapısal tüketimidir. Palette storage'ın bütün hücrelerinin global ID çözümü ve ilk sorgulanabilir section cache'i [ayrı P08.a/b kaydında](client-world-state-conformance.md) bulunur. Işık nibble değerleri, heightmap semantiği, registry ID çözümü, block entity world konumu, load/update/unload sırası, dimension/respawn ve render açık kalır. Başarılı canlı parser, oynanabilir istemci veya Java ile tam dünya eşitliği anlamına gelmez.

Sözleşme ayrıntıları [sözleşme bölümündedir](#sözleşme). Önceki login/configuration/play bağlantı kanıtı [vanilla oyun oturumu kaydında](game-connection-conformance.md), Pumpkin koşusu ise [ayrı kayıtta](game-connection-conformance.md) tutulur. Bu test kullanıcının istediği üzere resmi vanilla sunucusuyla yapıldı; Pumpkin eksikleri bu baseline'a karıştırılmadı.
