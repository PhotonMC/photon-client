# P08.e-entity-remove — [server-conformance] ClientWorld entity removal state

Bu kayıt, vanilla sunucusundan gelen `ClientboundRemoveEntitiesPacket`
akışının Rust `ClientWorld` içindeki entity state'inden silinmesini kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundRemoveEntitiesPacket.java:10-43`, entity ID listesini
`FriendlyByteBuf.readIntIdList` ile okur. `FriendlyByteBuf.java:146-160` wire
sırasını count VarInt ve count adet entity ID VarInt olarak tanımlar; packet'in
play dispatch ID'si `77`'dir (`GameProtocols.java:210`).

`ClientPacketListener.java:776-790`, listedeki her ID için mevcut entity'yi
arar; entity yoksa hiçbir işlem yapmaz, varsa `ClientLevel.removeEntity` ile
discard eder. `ClientLevel.java:522-537`, entity deposundan aynı ID'yi kaldırır
ve entity'nin client removal callback'ini çalıştırır.

## Sözleşme

- `REMOVE_ENTITIES` gövdesi count VarInt ve ardından entity ID VarInt listesini
  taşır. ID'ler entity add paketindeki global ID alanlarıyla eşleşir.
- `ClientWorld`, listedeki her mevcut ID'yi global entity map'inden kaldırır;
  bulunmayan ID'ler sessizce atlanır. Gözlem, toplam listedeki ID sayısını ve
  world'den gerçekten kaldırılan kayıt sayısını ayrı tutar.
- Bir entity'nin chunk içinde bulunması remove işlemi için ön koşul değildir;
  entity deposu chunk section kopyasından ayrıdır.
- Entity'nin passenger/vehicle özel durumu, removal reason callback'leri,
  chunk unload ile otomatik temizleme, dimension/respawn ve entity tick state'i
  bu dilimin dışındadır.

## Rust doğrulaması

`parse_remove_entities_packet`, Java count/ID listesini tüketir, negatif count
ile boş Java listesini korur ve trailing byte'ları reddeder. `WorldEntityRemove`
probe modu her packet'i parse eder, listedeki mevcut entity'leri
`ClientWorld`'den siler ve uygulanan sayıyı observation'a yazar.

`ClientWorld` unit testi aynı ID'nin eklenip silinmesini, count'un sıfıra
inmesini ve ikinci silmenin false dönmesini kontrol eder. Remove probe'u ayrıca
en az bir kaldırılan ID'nin daha önce add observation'ında görülmüş ve probe
sonunda world sorgusunda bulunmuyor olmasını zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken vanilla konsolunda iki kontrollü armor stand silindi:

```text
cargo run --locked --offline --example world_entity_remove_probe -- 127.0.0.1:25568 EntityRemove
summon minecraft:armor_stand 25 81 -36
kill @e[type=minecraft:armor_stand]
```

Gerçek çalıştırmada probe bağlantısı summon ve kill komutlarından önce açık
tutuldu; vanilla konsolu `Summoned new Armor Stand` ve `Killed 2 entities`
bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `remove_entities_packet_count` | `5` |
| ilk packet ID listesi | `[28]` |
| `remove_entities_applied_count` | `5` |
| `world_entity_count` | `68` |
| `world_chunk_count / world_section_count` | `157 / 3768` |
| `received_packet_count` | `12820` |

Probe başarıyla tamamlandığı için en az bir remove ID'sinin önceki add
gözlemlerinde mevcut olduğu ve işlem sonunda `ClientWorld`'de bulunmadığı da
doğrulandı. Bu sonuç vanilla remove-entities wire akışının Rust world
kopyasından silindiğini gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Passenger/vehicle bağları, removal reason ve callback semantiği, chunk unload
sırasında entity temizliği, entity metadata/equipment, dimension/respawn,
entity tick/fizik, remote interpolation, render state'i ve oynanabilir
istemciye bağlantı açık kalır. Bu raw entity lifecycle state'i
`server-conformance` ilerlemesidir; P08'in `client-parity` kapanışı değildir.
