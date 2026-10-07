# P08.c-block-entity — [server-conformance] ClientWorld block entity state

Bu kayıt, vanilla sunucusundan gelen chunk içi block entity listesinin ve
bağımsız `BLOCK_ENTITY_DATA` paketlerinin Rust `ClientWorld` içindeki
sorgulanabilir block entity state'ine uygulanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundLevelChunkPacketData.java:27-66,99-110,120-140` chunk gövdesinin
section verisinden sonra block entity kayıtlarını okur. Her kayıt packed X/Z
yerel koordinat byte'ı, signed short Y, block entity type registry VarInt'i ve
nullable NBT kökü taşır; X/Z chunk taban koordinatıyla dünya koordinatına
genişletilir. `ClientboundBlockEntityDataPacket.java:16-64` bağımsız pakette
packed `BlockPos` long'u, block entity type registry ID'si ve trusted compound
tag sırasını kullanır. Bu paket play dispatch tablosunda ID `6`'dır
(`GameProtocols.java:137-141`, `GamePacketTypes.java:14-18`).

`ClientPacketListener.java:850-861` chunk kopyası hazırlanırken chunk içi
block entity kayıtlarını uygular; `ClientPacketListener.java:1471-1485`
bağımsız paketi world position/type üzerinden mevcut block entity'ye yükler.
Sunucu tarafında `ChunkHolder.java:195-235`, değişen block state block entity
taşıyorsa ilgili update packet'ini aynı oyuncu listesine yayınlar.
`LevelChunk.java:364-405` chunk içi block entity deposunu, `BlockEntity.java:41-56,157-179,258-270`
type/position ve update tag sözleşmesini, `BlockEntityTypes.java:20-34` ise
registry type kimliklerini sağlar.

## Sözleşme

- Chunk packet'indeki packed X/Z değeri, chunk koordinatının 16 ile
  çarpılmış tabanına eklenir; Y signed short olarak korunur.
- NBT kökü null ise kayıt tag'siz tutulur. Compound kökü varsa decoder'ın
  tükettiği ham NBT byte aralığı saklanır ve `tag_length` gözlem alanına
  yazılır.
- Chunk kopyası oluşturulurken o column'ın önceki block entity kayıtları
  sıfırlanır. Yalnızca aynı chunk column'ına ait ve yüklü kayıtlar
  `ClientWorld`'e uygulanır.
- Bağımsız ID `6` paketi mevcut yüklü chunk'a `(x,y,z)` anahtarıyla eklenir
  veya aynı konumdaki type/tag state'ini değiştirir. Chunk yüklü değilse
  packet gözlemi korunur fakat world application sonucu `false` olur.
- Chunk unload, block entity map'ini de chunk ile birlikte kaldırır.
- Type registry ID'si şimdilik opaque integer'dır. Block entity'nin semantic
  NBT alanları, inventory item stack'leri, renderer state'i ve block state ile
  type geçerlilik kontrolü bu dilimin kapsamı dışındadır.

## Rust doğrulaması

`parse_chunk_packet`, packed yerel koordinatları ve nullable/compound NBT
köprüsünü tüketip `BlockEntityObservation` kayıtlarını üretir.
`parse_block_entity_data_packet`, packed `BlockPos`, type registry ID'si,
compound NBT ve trailing payload kontrolünü yapar. Birim testleri negatif
packed position, chunk içi yerel koordinat, null tag, compound tag ve trailing
byte hatasını kontrol eder.

`ClientWorld` block entity map'ini chunk replacement ve unload ile birlikte
tutar. Birim testi negatif dünya koordinatında ekleme, aynı konumda update,
type/tag sorgusu ve unload temizliğini kontrol eder. İki canlı probe, chunk içi
ve bağımsız server packet yollarının gerçekten world query'ye işlendiğini
ölçer; yalnız decoder çıktısı başarı sayılmaz.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

İlk chunk içinde kayıt üretmek için hedef chunk force-load edilip chest
yerleştirildi:

```text
forceload add 25 -36
setblock 25 80 -36 minecraft:chest
cargo run --locked --offline --example world_block_entity_probe -- 127.0.0.1:25568 BEInitial
```

Bağımsız paket yolu için aynı hedefte spawner block entity oluşturulurken
probe bağlantısı açık tutuldu:

```text
cargo run --locked --offline --example world_block_entity_update_probe -- 127.0.0.1:25568 BESpawner
setblock 25 80 -36 minecraft:air
setblock 25 80 -36 minecraft:spawner
```

Gözlemler:

| Gözlem | Chunk içi liste | Bağımsız `BLOCK_ENTITY_DATA` |
| --- | ---: | ---: |
| kayıt / paket sayısı | `4` kayıt | `2` paket |
| world'e uygulanan | `4` | `2` |
| ilk hedef koordinatı | `(25,80,-36)` | `(25,80,-36)` |
| ilk type registry ID / tag uzunluğu | `1 / 0` | `9 / 124` |
| son hedef query | — | `Some((9, Some(147)))` |
| world block entity sayısı | `4` | `12` |
| world chunk / section sayısı | `23 / 552` | `157 / 3768` |
| alınan packet sayısı | `161` | `12580` |

Server konsolu iki `setblock` değişikliğini başarıyla bildirdi; Rust probe
sonunda hedef spawner kaydını `ClientWorld` içinden sorguladı. Chest üzerinde
tek başına `data merge` komutu sunucu tarafında başarılı olsa da bu block
entity türü bağımsız update packet'i üretmedi; bu sonuç başarı kanıtına
katılmadı. Spawner'ın vanilla update packet yolu ile alınan iki packet'in
ikisi de world'e uygulandı. Bu sonuç, vanilla'nın block entity wire state'inin
Rust world kopyasına taşındığını gösterir; Java istemcisiyle parity iddiası
değildir.

## Açık kapsam

NBT semantic alanlarının type'a göre yüklenmesi, chest item listeleri ve
container state'i, block entity'nin block state değişiminde kaldırılması veya
type mismatch davranışı, yüklenmemiş chunk için update kuyruğu, dinamik
dimension/respawn, entity deposu/yaşam döngüsü ve block entity renderer state'i
açık kalır. Ham type/tag state'ini sorgulamak oynanabilir istemci veya tam
Java client parity kanıtı değildir.
