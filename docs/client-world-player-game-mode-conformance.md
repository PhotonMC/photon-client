# P09.ah — GameType ve local player game-mode state — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 clientbound game-event akışındaki
`CHANGE_GAME_MODE` dalını ve login/respawn spawn-info içindeki oyun modu değerinin
sorgulanabilir `ClientWorld` local player state'ine uygulanmasını taşır. Java
oracle ile Rust adapter karşılaştırması `client-parity` sınıfındadır. Resmi
vanilla sunucu koşusu aşağıda ayrı `server-conformance` adjunct'ı olarak
tutulur.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `network/protocol/game/ClientboundGameEventPacket.java:9-76`, packet'ın
  unsigned event byte'ı ve big-endian `float` parametresini, `CHANGE_GAME_MODE`
  event kimliğini `3` olarak ve codec tüketim sırasını tanımlar.
- `client/multiplayer/ClientPacketListener.java:490-542,1247-1336`, login ve
  respawn sonrasında `spawnInfo.gameType()` değerini local game mode'a uygular.
- `client/multiplayer/ClientPacketListener.java:1532-1546`, game-event
  parametresini `Mth.floor(param + 0.5F)` ile tamsayıya çevirir ve yalnızca
  `CHANGE_GAME_MODE` dalında local game mode'u günceller.
- `client/multiplayer/MultiPlayerGameMode.java:82-108`, local mode'un
  başlangıçta survival olduğunu ve mode değişiminde player abilities'in yeniden
  güncellendiğini belirler.
- `world/level/GameType.java:16-119`, dört mode ID/name değerlerini, geçersiz
  ID'nin survival fallback'ini, `isCreative`/`isSurvival` sınıflandırmasını ve
  `updatePlayerAbilities` alan atamalarını tanımlar.
- `network/protocol/game/GamePacketTypes.java:45` ve
  `network/protocol/game/GameProtocols.java:170-172`, game-event packet'ının
  play clientbound protocol ID'sini `38` olarak sabitler.

## Davranış sözleşmesi

- Game-event gövdesi tam olarak bir event byte'ı ve bir `f32` parametresidir;
  eksik veya trailing byte kabul edilmez.
- Event ID `3` için Java, `floor(param + 0.5F)` sonucunu `GameType.byId`'ye
  verir. ID `0..3` sırasıyla survival, creative, adventure ve spectator'dır;
  aralık dışı ID survival'a döner.
- Survival ve adventure `isSurvival` olarak sınıflanır; adventure ve spectator
  block placing'i kısıtlar. Creative yalnız invulnerable, may-fly ve instabuild
  alanlarını açar ve mevcut flying değerini korur. Spectator flying'i açar;
  survival/adventure bu dört ability alanını kapatır. Flying/walking speed
  değerleri mode güncellemesinde korunur.
- Local `mayBuild`, survival ve creative için true; adventure ve spectator
  için false olur. Clientbound abilities packet'ı `mayBuild` taşımadığından,
  packet uygulanırken mevcut mode'dan türeyen değer korunur.
- Login ve respawn spawn-info game type'ı aynı Java ID mapping'iyle local
  player mode'una uygulanır. Yeni local player oluşturulunca mode varsayılan
  survival'dır; spawn-info veya game-event sonraki state'i belirler.
- Event ID `3` dışındaki game-event'ler parse edilir, fakat bu dilimde local
  game-mode state'ini değiştirmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`shape/player-game-mode/change-game-mode` kaydı dokuz özgün gözlem taşır:

| Gözlem | Parametre bitleri | Java sonucu |
| --- | --- | --- |
| `survival` | `00000000` | survival, mayBuild true |
| `creative` | `3f800000` | creative, invulnerable/mayFly/instabuild true |
| `creative-preserves-existing-flying` | `3f800000` | creative, önceden true olan flying ve iki speed korunur |
| `adventure` | `40000000` | adventure, isSurvival true, mayBuild false |
| `spectator` | `40400000` | spectator, flying/mayFly/invulnerable true, mayBuild false |
| `rounds-to-creative` | `3fbeb852` | `floor(param + 0.5F)` sonrası creative |
| `exact-half-rounds-up` | `3fc00000` | eşik sonrası adventure |
| `high-id-falls-back-to-survival` | `40666666` | geçersiz yüksek ID sonrası survival |
| `negative-id-falls-back-to-survival` | `bf028f5c` | geçersiz negatif ID sonrası survival |

Java oracle gerçek `ClientboundGameEventPacket.STREAM_CODEC` codec'ini,
`Mth.floor` sonucunu ve gerçek `GameType.updatePlayerAbilities` yolunu kullanır;
beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`53e7ad660a1485f68ebb6cff6e09a2d7eb60315b4e37cb7ccb92ef5db1c738ea`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2949** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`parse_game_event_packet` protocol ID `38` gövdesini event byte'ı ve big-endian
`f32` olarak çözer. `ClientPlayerGameMode`, Java ID fallback'ini, parametre
yuvarlamasını, mode sınıflandırmasını ve mevcut `ClientPlayerAbilities`
üzerindeki güncellemeyi uygular. `ClientWorld` login/respawn spawn-info,
clientbound abilities packet ve game-event yollarını local player state'ine
bağlar; connection trace event'in uygulanıp uygulanmadığını ve mode state'ini
gözlemler. Behavior adapter aynı production parser ve world-mode helper'larını
kullanır.

Debug adapter SHA-256 değeri
`f088d8b89f9d891f376ea55f02496fed62815f0bdf2f8aa8a10c36ccbde4b568` ile
**2949/2949** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`ac797173d677df6e2f4274df19320bcf93fd97fb5a673d01dd851ad18c1ac2b1` ile
aynı capture üzerinde **2949/2949** eşleşti; fark `null` kaldı.

Dilim sırasında çalıştırılan hedef kontroller:

```text
cargo test --locked --offline local_player_game_mode -- --nocapture
cargo test --locked --offline game_event -- --nocapture
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
```

## Vanilla server-conformance adjunctı

Mojang vanilla 26.2 dedicated server JAR'ı
`823e2250d24b3ddac457a60c92a6a941943fcd6a` SHA-1'iyle doğrulandı. Sunucu,
offline ve yalnızca `127.0.0.1:25568` üzerinde, seed `776`, creative,
peaceful, view/simulation distance `5` ayarlarıyla geçici
`/tmp/voxel-vanilla-26.2-test` dizininde çalıştırıldı. JAR, dünya ve loglar
depoya alınmadı.

```text
cargo run --locked --offline --example connection_probe -- 127.0.0.1:25568 GameModeProbe2
```

Gözlenen Rust trace özeti:

```text
connection=ok
play_login=true
player_abilities_observations=[PlayerAbilitiesObservation { packet: PlayerAbilitiesPacket { invulnerable: true, flying: false, can_fly: true, can_instabuild: true, flying_speed: 0.05, walking_speed: 0.1 }, applied_to_world: true }]
game_event_observations=[GameEventObservation { packet: GameEventPacket { event_id: 13, param: 0.0 }, applied_to_world: false, local_game_mode: Some(Creative) }]
world_chunk_count=24
world_section_count=576
```

Resmi sunucunun kısa login oturumunda aldığı game-event `LEVEL_CHUNKS_LOAD_START`
event'idir; `CHANGE_GAME_MODE` event'i bu oturumda üretilmediği için canlı
koşu bu dal için sentetik parity capture'ının yerine geçirilmez. Koşu, vanilla
server'ın Rust paketini kabul ettiğini, login game type'ının local world state'e
uygulandığını ve bilinmeyen game-event dalının state'i bozmadan tüketildiğini
gösterir. `server-conformance` sonucu Java istemcisiyle aynı girdide davranış
eşitliği veya oynanabilir istemci kanıtı değildir.

## Açık kapsam

Game-mode state'inin movement tick, block/eşya action kısıtları, uçuş/elytra,
input ve kamera davranışlarına bağlanması; serverbound game-mode değişim
akışları; tam respawn sonrası oyuncu state'i ve diğer game-event yan etkileri
açık kalır. Bu kayıt mode mapping, abilities update ve sorgulanabilir local
player world state'ine uygulama ile sınırlıdır.
