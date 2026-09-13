# P08.an — [server-conformance] Entity sleeping position semantic state

Bu kayıt, Java `LivingEntity` uyku yatağı konumu alanının Rust
`ClientWorld` state'ine sınırlı ve sorgulanabilir biçimde uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:187-189`, `SLEEPING_POS_ID` alanını ID `14` ve
`EntityDataSerializers.OPTIONAL_BLOCK_POS` ile tanımlar. `LivingEntity.java:322-330`,
alanı `Optional.empty()` ile başlatır. `LivingEntity.java:3764-3777`, getter,
setter, temizleme ve `isSleeping()` davranışını bu synchronized değere bağlar.

`LivingEntity.java:3780-3799`, yatak başlangıcında pose'u `SLEEPING`e çevirir,
entity konumunu yatağa göre ayarlar, sleeping position'ı yazar ve hareketi
sıfırlar. `LivingEntity.java:3523-3529`, client tarafında ID `14` değişince
oyuncu/entity konumunun yatağa taşınması için aynı alanı kullanır.

`EntityDataSerializers.java:94-96,173-175`, `OPTIONAL_BLOCK_POS` serializer'ının
ID `11` olduğunu ve `BlockPos.STREAM_CODEC` üzerine kurulduğunu gösterir.
`ByteBufCodecs.java:373-385`, Optional değerinin bir boolean presence byte'ı
taşıdığını, değer varsa iç codec'i çalıştırdığını tanımlar.
`FriendlyByteBuf.java:385-400` ve `BlockPos.java:38-44,74-115`, iç değerin
sekiz byte big-endian packed long olarak yazılıp `x`, `y`, `z` alanlarına
ayrıştırıldığını doğrular.

Oyuncu yatağa `Player.java:1335-1339` üzerinden başlar; server tarafındaki
bed kontrolü başarılı olduğunda `LivingEntity.startSleeping` ortak synchronized
alanları günceller.

## Sözleşme

- Geçerli metadata içinde ID `14` ve `OPTIONAL_BLOCK_POS` serializer ID `11`
  varsa presence byte'ı okunur. Presence `0` boş sleeping position'dır;
  presence sıfır değilse sonraki sekiz byte packed `BlockPos` olarak çözülür.
- Değer `ClientEntity::sleeping_pos` veya
  `ClientPlayerState::sleeping_pos` üzerinden sorgulanır. Dolu değer
  `is_sleeping()` sonucunu true yapar; boş değer önceki konumu temizler.
- Metadata yalnız gönderilen alanları günceller. ID `14` içermeyen sonraki
  packet önceki konumu korur; yanlış serializer, eksik packed long veya
  metadata EOF'si olmayan kayıt state'i değiştirmez.
- Local player metadata'sı aynı konumu player world kopyasına uygular.
  Respawn `KEEP_ENTITY_DATA` maskesi entity data'yı koruduğunda konum korunur;
  boş keep maskesi yeni player varsayılanı olan boş değere döner.
- Bu dilim sleeping pose'unu, yatağa konumlandırma transformunu, bed facing
  çözümünü, bed occupancy güncellemesini veya gerçek uyku/wake lifecycle'ını
  yürütmez. Bunlar aynı packet akışında ayrı state ve davranış kapsamıdır.

## Rust doğrulaması

`skip_metadata_value`, serializer ID `11` için Optional presence byte'ını ve
varsa sekiz byte packed position'ı tüketir. `sleeping_pos_from_metadata`, ID
`14` kaydının serializer ve payload'ını doğrular; başarısız codec durumunda
önceki world state'i korur.

`ClientEntity` ve `ClientPlayerState` `Option<BlockPos>` alanını tutar ve
`sleeping_pos()` ile `is_sleeping()` getter'larını sunar. World unit testleri
packed negatif/pozitif koordinatları, boş başlangıcı, sonraki metadata alanını,
kısmi metadata korunmasını, boş Optional temizliğini, yanlış serializer'ı ve
eksik payload'ı kontrol eder. Local player testi değerin
`KEEP_ENTITY_DATA` ile korunmasını ve boş keep maskesiyle temizlenmesini
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Oyuncu vanilla sunucusuna bağlandı. Sunucunun gerçek spawn konumu
`(26.5, 63.0, -41.5)` ölçüldü; oyuncunun altındaki foot bed bloğu `(26, 62,
-42)`, north-facing bed head bloğu `(26, 62, -43)` olarak hazırlandı ve dünya
geceye alındı. Hazırlık penceresinden sonra Rust probe'u empty main hand ile
foot bloğuna `use_item_on` gönderdi:

```text
cargo run --locked --offline --example world_entity_sleeping_position_probe -- 127.0.0.1:25568 P08Sleep3
time set night
setblock 26 62 -42 minecraft:red_bed[facing=north,part=foot,occupied=false]
setblock 26 62 -43 minecraft:red_bed[facing=north,part=head,occupied=false]
```

Vanilla oyuncu etkileşimini kabul etti; Rust world metadata akışını uygulayıp
başarıyla tamamlandı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `121` |
| `sleeping_pos` | `BlockPos { x: 26, y: 62, z: -43 }` |
| `is_sleeping` | `true` |
| `pose` | `Sleeping` |
| `local_player_position` | `(26.5, 63.0, -41.5)` |
| `world_entity_count` | `39` |
| `world_chunk_count` | `46` |
| `received_packet_count` | `5343` |

Bu kayıt ID `14` Optional payload'ının vanilla sunucusundan geldiğini ve
packed position'ın `ClientWorld` state'ine çözüldüğünü gösterir. Bu sonuç Java
istemcisiyle parity iddiası değildir.

## Açık kapsam

Sleeping pose/dimension/collision, client-side yatağa position snap'i, bed
facing ve occupancy, wake/respawn lifecycle, villager/diğer living entity
uyku dalları, bed interaction sonucu ve `client-parity` karşılaştırması açık
kalır.
