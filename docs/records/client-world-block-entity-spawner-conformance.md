# P08.s — [server-conformance] ClientWorld spawner semantic state

Bu kayıt, vanilla mob spawner block entity'sinin bağımsız update paketindeki
NBT alanlarının Rust `ClientWorld` içinde sorgulanabilir semantic state'e
çevrilmesini kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`BaseSpawner.java:39-59,88-107,194-217` spawner'ın yedi sayısal state alanını,
çalışma sırasında `spawnDelay` değişimini ve yükleme varsayılanlarını tanımlar.
`BaseSpawner.java:220-228` bu alanları update tag'inde kısa sayısal değerler
olarak yazar. `Delay` yüklenirken kısa, diğer alanlar tamsayı olarak okunur;
Java `ValueInput` numeric tag'leri türden bağımsız sayısal dönüşümle tüketir.

`SpawnerBlockEntity.java:41-70` block entity yükleme/kaydetme yolunu, bağımsız
update packet'ini ve client update tag'inden `SpawnPotentials` alanının
çıkarılmasını verir. `ClientboundBlockEntityDataPacket.java:16-39,41-65`
packed konum, block entity type registry holder'ı ve trusted compound tag'i
taşır. `ClientPacketListener.java:1471-1489` yalnız konumda mevcut ve packet
type'ıyla eşleşen block entity'ye tag'i yükler. `BlockEntityTypes.java:89`
`MOB_SPAWNER` kaydını kurar; vanilla testinde bu registry ID'si `9`'dur.

## Sözleşme

- Block entity type registry ID'si `9` ve geçerli bir compound tag varsa yedi
  alan `ClientSpawnerState` olarak saklanır. Raw tag önceki P08.c sözleşmesinde
  olduğu gibi korunur.
- Alanlar `Delay`, `MinSpawnDelay`, `MaxSpawnDelay`, `SpawnCount`,
  `MaxNearbyEntities`, `RequiredPlayerRange` ve `SpawnRange` sırasıyla Java
  varsayılanları `20`, `200`, `800`, `4`, `6`, `16` ve `4` ile yüklenir.
- Java'nın `Delay` için kısa, diğer alanlar için tamsayı numeric-tag dönüşümü
  izlenir. Eksik veya numeric olmayan alanlar ilgili Java varsayılanına döner;
  bilinmeyen alanlar yok sayılır.
- Aynı konuma gelen yeni update önceki typed state'i değiştirir. Type `9`
  olmayan block entity'ler spawner state üretmez; chunk unload raw ve typed
  block entity kaydını birlikte kaldırır.

## Rust doğrulaması

`ClientBlockEntity::from_wire`, type `9` için compound NBT'yi ayrıştırıp
`spawner_state()` üzerinden yedi alanı açar. `ClientWorld`'ün mevcut chunk içi
ve bağımsız `BLOCK_ENTITY_DATA` uygulama yolları aynı constructor'ı kullandığı
için typed state her iki kaynaktan da sorgulanabilir.

Birim testi kısmi bir tag'de Java varsayılanlarını, kısa dönüşümünü ve yanlış
numeric türde varsayılana dönüşü kontrol eder. `world_block_entity_spawner_state_probe`
bağımsız vanilla update paketini hedef `(25,80,-36)` kaydından alır ve typed
state'in world query'sinden okunmasını zorunlu tutar.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, peaceful, `view-distance=5`,
`simulation-distance=5`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken hedefteki eski block entity temizlenip spawner yeniden
oluşturuldu:

```text
cargo run --locked --offline --example world_block_entity_spawner_state_probe -- 127.0.0.1:25568 P08sJ
setblock 25 80 -36 minecraft:air
setblock 25 80 -36 minecraft:spawner
```

İki bağımsız vanilla oturumundaki sonuçlar:

| Gözlem | `P08sJ` | `P08sK` |
| --- | ---: | ---: |
| `play_login / player_loaded_sent` | `true / true` | `true / true` |
| world'e uygulanan update sayısı | `3` | `2` |
| block entity type ID | `9` | `9` |
| `Delay` | `331` | `532` |
| `Min/MaxSpawnDelay` | `200 / 800` | `200 / 800` |
| `SpawnCount` | `4` | `4` |
| `MaxNearbyEntities` | `6` | `6` |
| `RequiredPlayerRange / SpawnRange` | `16 / 4` | `16 / 4` |
| world block entity / chunk / section sayısı | `18 / 157 / 3768` | `18 / 157 / 3768` |
| alınan packet sayısı | `9762` | `9177` |

`Delay`, spawner'ın vanilla server tick'i içinde yeniden zamanlanabildiği için
iki oturumda değişti; sabit değer olarak kullanılmadı. Her iki oturumda da
type `9` compound update tag'i Rust `ClientWorld` state'ine uygulandı ve
varsayılan alanlar typed query üzerinden doğrulandı. Sonuç vanilla wire
uyumluluğunu ve world uygulamasını gösterir; Java client eşitliği iddiası
değildir.

## Açık kapsam

`SpawnData` içindeki entity type ve entity özel alanlarının semantic çözümü,
`SpawnPotentials`, spawner client tick/display rotation, spawner renderer,
block state/type doğrulaması, diğer block entity türlerinin semantic alanları
ve container item listeleri açık kalır. Hopper chunk kayıtlarında vanilla
`Items` tag'ini taşımadı; hopper item state'i menu content paketinde ayrı bir
local player container state olarak doğrulandı. Bu sonuç block entity item
listesi veya oynanabilir renderer parity'si değildir.
