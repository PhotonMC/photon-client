# P08.e-entity-add — [server-conformance] ClientWorld entity add state

Bu kayıt, vanilla sunucusundan gelen `ClientboundAddEntityPacket` akışının
Rust `ClientWorld` içinde sorgulanabilir entity state'ine uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundAddEntityPacket.java:17-31,95-120,132-155` alanları entity ID
VarInt, UUID, entity type registry VarInt, üç konum double'ı, sıkıştırılmış
hareket vektörü, üç rotation byte'ı ve data VarInt sırasıyla taşır. Play
dispatch tablosunda bu packet'in clientbound ID'si `1`'dir.

`ClientPacketListener.java:565-590`, paketten entity türünü çözüp entity
nesnesini oluşturur ve `ClientLevel.addEntity` yoluna verir. `ClientLevel.java:522-537`,
aynı entity ID'si zaten varsa önceki kaydı kaldırır ve yeni entity'yi
ID anahtarıyla depolar. Bu, ekleme paketinin chunk section kopyasından bağımsız
olan global entity deposu dalıdır.

## Sözleşme

- `ADD_ENTITY` gövdesi entity ID, type registry ID'si, üç f64 konum ve data
  VarInt alanlarını tüketir; UUID, sıkıştırılmış hareket ve rotation alanları
  da wire sınırını doğrulamak için tüketilir ve başlangıç movement değeri
  [entity motion kaydında](client-world-entity-motion-conformance.md) world state'ine taşınır.
- `ClientWorld`, entity'yi entity ID ile anahtarlanan global map'e ekler.
  Aynı ID tekrar gelirse kayıt değiştirilir ve benzersiz entity sayısı artmaz.
- Konumun f64 bitleri korunur; `entity_at` sorgusu type ID, kesin konum ve
  data değerini verir.
- Entity ekleme, ilgili chunk'ın `ClientWorld` içinde bulunmasını gerektirmez.
  Packet gözlemi ve world application sonucu ayrı izlenir.
- UUID state'i [ayrı P08.e UUID kaydında](client-world-entity-uuid-conformance.md)
  ele alınır. Başlangıç hareketi ve motion packet state'i [ayrı motion kaydında](client-world-entity-motion-conformance.md)
  ele alınır. Üç rotation değeri,
  metadata/equipment paketleri, remove packet'leri ve entity tick/interpolation
  state'i bu dilimin dışındadır.

## Rust doğrulaması

`parse_add_entity_packet`, Java alan sırasındaki bütün wire alanlarını tüketir;
UUID'yi ve başlangıç `LpVec3` hareketini ayrıştırır ve trailing byte'ları
reddeder. `WorldEntityAdd` probe modu her başarılı add paketini `EntityInput`
olarak `ClientWorld`'e uygular ve observation üzerinde uygulama sonucunu
işaretler. UUID'nin world state'ine taşınması [ayrı UUID kaydında](client-world-entity-uuid-conformance.md), başlangıç ve canlı motion state'i
[motion kaydında](client-world-entity-motion-conformance.md) doğrulanır.

`ClientWorld` unit testi aynı ID ile iki entity eklemesini, map boyutunun sabit
kalmasını ve son type/position/data state'inin sorgulanmasını kontrol eder.
Entity add probe'u ise ilk entity gözleminin world map'ten sorgulanmasını
zorunlu tutar; yalnız parser'ın paket tüketmesi başarı sayılmaz.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken sunucu konsolunda kontrollü bir entity oluşturuldu:

```text
summon minecraft:armor_stand 25 81 -36
cargo run --locked --offline --example world_entity_add_probe -- 127.0.0.1:25568 EntityAdd
```

Vanilla konsolu `Summoned new Armor Stand` bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `add_entity_packet_count` | `72` |
| `add_entity_applied_count` | `72` |
| ilk entity | `id=25, type_id=110, (34.371204360275335,58.92584090886237,19.360545755048946), data=0` |
| `world_entity_count` | `70` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `14856` |

Canlı oturumda parser'ın yakaladığı her add kaydı world'e uygulandı; map
sayısının paket sayısından düşük olması ID anahtarının tekrar gelen entity'yi
değiştirebildiğini gösterir. Bu sonuç vanilla'nın entity-add wire akışının Rust
world kopyasına taşındığını gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Entity type registry'nin semantic çözümü, metadata/equipment/status paketleri,
`REMOVE_ENTITIES` yaşam döngüsü, chunk
unload ile entity temizliği, dimension/respawn, entity tick/fizik, remote
interpolation, render state'i ve `ClientWorld`'ün oynanabilir istemciye
bağlanması açık kalır. Bu raw entity state'i `server-conformance` ilerlemesidir;
P08'in `client-parity` kapanışı değildir.
