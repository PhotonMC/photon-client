# P09.ag — ClientboundPlayerAbilities packet ve local player state — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2'nin clientbound `player_abilities` packet'ını
decode edip local player abilities state'ine uygulama yolunu taşır. Kanıt sınıfı
`client-parity`'dir. Canlı vanilla sunucu koşusu aşağıda ayrı
`server-conformance` adjunct'ı olarak tutulur.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `network/protocol/game/ClientboundPlayerAbilitiesPacket.java:9-40,75-97`,
  packet'ın bir flag byte'ı ve iki `float` alanı taşıdığını, dört flag maskesini
  ve getter sırasını belirler.
- `client/multiplayer/ClientPacketListener.java:2085-2093`, gelen değerleri
  `minecraft.player` içindeki `Abilities` state'ine sırasıyla yazar.
- `world/entity/player/Abilities.java:7-37`, yeni abilities state'inin dört
  boolean için false, flying speed için `0.05F` ve walking speed için `0.1F`
  varsayılanlarını tanımlar.
- `network/protocol/game/GamePacketTypes.java:67` ve
  `network/protocol/game/GameProtocols.java:191-198`, packet type'ının play
  clientbound codec kaydındaki yerini belirler. Bu kayıt için protocol ID `64`
  olarak sabitlenmiştir.

## Davranış sözleşmesi

- İlk byte içindeki `0x01`, `0x02`, `0x04` ve `0x08` bitleri sırasıyla
  invulnerable, flying, may-fly ve instabuild alanlarını seçer. Diğer bitler
  okunur fakat state'e ek bir anlam vermez.
- Flag byte'ından sonra flying speed ve walking speed dört byte IEEE-754
  `float` olarak, Java wire sırasıyla okunur. Raw bitler korunur; `-0.0` ve
  NaN gibi değerler sayısal olarak normalize edilmez.
- Client listener altı packet alanını local player abilities state'ine yazar;
  packet bu dilimde `mayBuild` veya game-mode değerini taşımaz.
- Local player world kopyasında packet uygulanmadan önce varsayılan abilities
  değerleri vardır. Yeni local player oluşturulduğunda bu state yeni player'ın
  varsayılanlarına döner.
- Parser dokuz byte'lık packet gövdesini tam tüketir; eksik veya trailing byte
  kabul edilmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`shape/player-abilities/clientbound-state` kaydı altı özgün gözlem taşır:

| Gözlem | Flag durumu | Flying speed | Walking speed |
| --- | --- | --- | --- |
| `defaults` | tümü false | `3d4ccccd` | `3dcccccd` |
| `invulnerable-only` | yalnız invulnerable | `3d4ccccd` | `3dcccccd` |
| `flying-and-can-fly` | flying + may-fly | `3e4ccccd` | `3d4ccccd` |
| `instabuild-only` | yalnız instabuild | `3d4ccccd` | `3dcccccd` |
| `all-flags-and-unknown-bit` | dört alan true, bilinmeyen bitler yok sayılır | `3e4ccccd` | `3e99999a` |
| `raw-speed-bits` | flying + may-fly false | `80000000` | `7fc00001` |

Java oracle gerçek `ClientboundPlayerAbilitiesPacket.STREAM_CODEC` codec'ini
ve gerçek `Abilities` setter/getter yolunu kullanır; beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`b9c85175c53e2f91148ec1b3b6b05364b899a14c41315ef2b0c72bcbc141495a`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2948** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`parse_player_abilities_packet` protocol ID `64` gövdesini flag maskeleri ve
iki big-endian `f32` alanıyla çözer. `ClientPlayerAbilities` raw speed bitlerini
koruyarak local `ClientPlayerState` içinde saklar. Connection probe packet'ı
production parser üzerinden geçirir ve `ClientWorld` local player state'ine
uygular; local player yeniden oluşturulunca abilities varsayılan state'e döner.
Behavior adapter aynı production parser'ı kullanır.

Debug adapter SHA-256 değeri
`6e26ad9d5c2ce9964ee91b6cf8e3a92009ad8813358a6babc3f14062e017f7c4` ile
**2948/2948** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`07646b8e1ef68b82940edfb4f09b760ea181a556548f9113202da28fc56c4386` ile
aynı capture üzerinde **2948/2948** eşleşti; fark `null` kaldı.

Dilim sırasında çalışan temel kontroller:

```text
cargo test --locked --offline player_abilities -- --nocapture
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
cargo run --locked --offline --example connection_probe -- 127.0.0.1:25568 AbilitiesProbe2
```

Gözlenen Rust trace özeti:

```text
connection=ok
play_login=true
player_abilities_observations=[PlayerAbilitiesObservation { packet: PlayerAbilitiesPacket { invulnerable: true, flying: false, can_fly: true, can_instabuild: true, flying_speed: 0.05, walking_speed: 0.1 }, applied_to_world: true }]
player_loaded_sent=true
world_chunk_count=22
world_section_count=528
```

Ana binary ile aynı vanilla server üzerinde yapılan ek smoke koşusu da
başarılıdır:

```text
cargo run --locked --offline -- 127.0.0.1:25568 LiveAbilities 10 10
client_connection=ok
play_login=true
world_constructed=true
world_chunk_count=24
world_section_count=576
update_tags_packet_count=1
ticks_run=10
world_game_time=10
local_player_tick_count=10
rendered_frames=0
```

Bu koşu vanilla sunucusunun Rust oturumunu kabul ettiğini ve gerçek
clientbound abilities packet'ının parser ile world kopyasına uygulandığını
gösterir. `server-conformance` sonucu Java istemcisiyle aynı girdide davranış
eşitliği veya oynanabilir istemci kanıtı değildir.

## Açık kapsam

Packet state'inin local movement tick'inde flying/walking speed ve flying
kararlarına bağlanması; serverbound ability güncellemeleri; game-mode
dependent uçuş, elytra, input ve kamera davranışları açık kalır; mode ve
`mayBuild` state'i [P09.ah kaydında](client-world-player-game-mode-conformance.md)
bağlandı. Bu kayıt packet decode ve sorgulanabilir local player state'ine
uygulama ile sınırlıdır.
