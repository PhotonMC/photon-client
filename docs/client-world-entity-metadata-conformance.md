# P08.e-entity-metadata — [server-conformance] ClientWorld raw entity metadata

Bu kayıt, vanilla sunucusundan gelen `ClientboundSetEntityDataPacket`
akışının mevcut Rust `ClientWorld` entity'sine raw metadata state'i olarak
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundSetEntityDataPacket.java:11-52`, entity ID VarInt'inden sonra
metadata kayıtlarını ve `EOF_MARKER=255` sonlandırıcısını taşır.
`SynchedEntityData.java:216-245`, her kaydın unsigned byte alan ID'si, serializer
type VarInt'i ve serializer'a ait değer gövdesi sırasını verir. Serializer
registry'si `EntityDataSerializers.java:51-205` içinde kurulur; değer türleri
byte/int/float/string/component/item stack, pose, rotation ve registry bağlı
diğer codec'leri içerir. Play dispatch ID'si `GameProtocols.java:232` satırında
`99`'dur.

`ClientPacketListener.java:630-635`, packet entity'si world'de bulunuyorsa
`SynchedEntityData.assignValues` çağırır; bilinmeyen entity için packet'i
uygulamaz. Java'nın semantik entity field güncellemeleri bu serializer ve
entity type tanımlarına bağlıdır.

## Sözleşme

- Packet gövdesi entity ID VarInt'i ve sonu `255` olan metadata byte dizisini
  taşır. Ham slice, entity ID'den sonraki serializer/values/EOF byte'larını
  korur.
- Metadata yalnızca mevcut entity ID'sine uygulanır; world'de bulunmayan ID
  için observation tutulur fakat application sonucu false olur.
- Aynı entity için sonraki metadata packet'i önceki ham metadata state'ini
  değiştirir. Sorgu, marker dahil raw byte uzunluğunu verir.
- Bu dilim serializer type/value çözümlemesi, metadata ID'lerinin entity type'a
  göre doğrulanması ve `onSyncedDataUpdated` yan etkilerini uygulamaz.

## Rust doğrulaması

`parse_entity_metadata_packet`, entity ID'sini ayırır, Java EOF marker'ını
doğrular ve kalan gövdeyi raw byte olarak saklar. `WorldEntityMetadata` probe
modu gözlemi `ClientWorld` entity map'ine bağlar; uygulama sonucu ve metadata
uzunluğu ayrı kaydedilir.

`ClientWorld` unit testi metadata'nın mevcut entity'ye yazıldığını, olmayan
ID'nin reddedildiğini ve aynı ID ile entity replacement sonrasında eski
metadata'nın taşınmadığını kontrol eder. Probe, en az bir uygulanmış metadata
gözleminin world sorgusundan okunmasını zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken kontrollü armor stand oluşturuldu ve entity NBT'si
değiştirildi:

```text
cargo run --locked --offline --example world_entity_metadata_probe -- 127.0.0.1:25568 EntityMetadata
summon minecraft:armor_stand 25 81 -36
data merge entity @e[type=minecraft:armor_stand,limit=1,sort=nearest] {Invisible:1b}
```

Vanilla konsolu `Summoned new Armor Stand` ve `Modified entity data of Armor
Stand` bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `240` |
| `entity_metadata_applied_count` | `239` |
| ilk metadata | `id=33, length=10, applied=true` |
| ilk entity'nin son raw metadata query uzunluğu | `5` |
| `world_entity_count` | `74` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `14209` |

Bir packet mevcut entity map'inde karşılık bulmadığı için Java listener ile
uyumlu olarak uygulanmadı; uygulanan 239 kaydın en az biri world sorgusundan
okundu. Sonraki metadata paketinin aynı entity state'ini değiştirebildiği ilk
packet uzunluğu ile son query uzunluğunun farklı olmasından görülür. Bu sonuç
vanilla raw entity metadata wire akışının Rust world kopyasına taşındığını
gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Serializer türlerinin ve değerlerinin semantic decode edilmesi, metadata ID'si
ile entity type doğrulaması, custom name/component/item stack/pose/equipment
davranışları, `onSyncedDataUpdated` yan etkileri, tick/interpolation, render
state'i, dimension/respawn ve oynanabilir istemciye bağlantı açık kalır. Bu raw
metadata state'i `server-conformance` ilerlemesidir; P08'in `client-parity`
kapanışı değildir.
