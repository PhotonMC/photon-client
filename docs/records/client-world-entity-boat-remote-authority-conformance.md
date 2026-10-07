# P08.bk-entity-boat-authority — [server-conformance] Remote boat authority branch

Bu kayıt, client tarafında local instance authority sahibi olmayan bir boat'ın
tick dalını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust world kopyası
Mojang vanilla 26.2 sunucusundan gelen oak boat ve Zombie passenger state'i
üzerinde çalıştırıldı. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:210-251` içindeki `tick`, ortak `super.tick()` sonrasında
`isLocalInstanceAuthoritative()` sonucunu dallandırır. Authority varsa
`floatBoat`, client tarafında `controlBoat` ve ardından `move` çalışır;
authority yoksa `setDeltaMovement(Vec3.ZERO)` çağrılır. Bu dal client control,
float ve hareket adımlarını yürütmez.

`Entity.java:3587-3603`, client level'da local authority'nin
`isLocalClientAuthoritative()` üzerinden hesaplandığını gösterir. Bu değer
`getControllingPassenger()` tarafından seçilen living passenger'ın local
client olup olmadığına bağlıdır. `AbstractBoat.java:760-768` boat için ilk
passenger living entity'yi controlling passenger olarak seçer; base Entity
fallback'i boş olduğunda authority oluşmaz. `Player.java:1276-1283` local
client override'ının yalnız local Player için true olduğunu belirtir.

`ClientLevel.java:471-490`, kök vehicle'ın bir kez tick edilmesini ve bağlı
passenger'ın ayrı `rideTick` akışını tanımlar. Bu nedenle remote boat'ın
authority dalı, passenger input'ı veya local prediction state'i olmadan
sunucudan gelen boat hareketini yalnızca tüketir; burada kontrol input'ı
uygulamaz.

## Rust sözleşmesi ve test-first kanıtı

`ClientWorld::tick_entity_once`, pinned boat type'ı için ilk passenger ID'sinin
local Player olması koşulunu local instance authority olarak kullanır. Authority
yoksa boat motion'ı tam sıfıra çekilir; boat input snapshot'ı, `deltaRotation`,
yaw ve paddle state'i değiştirilmez. Authority olan boat'ın control ve
`IN_AIR` dalları P08.bh ve P08.bj kayıtlarında tutulur.

`remote_boat_tick_does_not_apply_client_control_or_motion` testi önce
uygulama yazılmadan çalıştırıldı ve sağ input'ın remote boat motion'ını
değiştirdiğini göstererek başarısız oldu. `boat_authority_uses_first_passenger_only`
testi de local Player ikinci sıradayken aynı control yolunun çalıştığını
göstererek first-passenger düzeltmesinden önce başarısız oldu. Authority ve
sıralama ayrımı eklendikten sonra iki test; konumun, rotation'ın,
`deltaRotation` değerinin ve paddle state'inin korunduğunu, motion'ın
sıfırlandığını doğrular. Bu kayıtla birlikte lib test corpus'u
`143 passed; 0 failed` sonucuna ulaştı.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusunun uzağında no-gravity oak boat ve no-AI/no-gravity,
invulnerable Zombie oluşturuldu. Zombie boat'ın ilk ve tek passenger'ı
olduğu için local Player boat'ın controlling passenger'ı değildir. Probe,
remote boat'a sağ input verip bir world tick çalıştırdı.

Fixture komutları:

```text
execute as P08Remote95 at @s run summon minecraft:oak_boat ~5 ~20 ~ {Tags:["P08BKR95"],NoGravity:1b,Rotation:[0.0f,0.0f]}
execute as P08Remote95 at @s run summon minecraft:zombie ~5 ~20 ~ {Tags:["P08BKP95"],NoAI:1b,NoGravity:1b,PersistenceRequired:1b,Invulnerable:1b,Rotation:[0.0f,0.0f]}
ride @e[type=minecraft:zombie,tag=P08BKP95,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08BKR95,limit=1,sort=nearest]
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_passenger_boat_remote_probe -- 127.0.0.1:25568 P08Remote95
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
local_player_id=22444
vehicle_id=22565
vehicle_type_id=89
passenger_id=22566
passenger_type_id=151
before_position=(20.5, 84.0, -28.5)
after_position=(20.5, 84.0, -28.5)
before_motion=(0.0, 0.0, 0.0)
after_motion=(0.0, 0.0, 0.0)
before_rotation=(0.0, 0.0, 0.0)
after_rotation=(0.0, 0.0, 0.0)
before_delta_rotation=0
after_delta_rotation=0
before_paddles=(false, false)
after_paddles=(false, false)
world_entity_count=58
world_chunk_count=157
received_packet_count=16304
```

Oak boat type `89`, Zombie type `151` ve local Player'ın passenger olmadığı
ilişki doğrulandı. Sağ input sonrasında Rust world kopyası konumu, rotation'ı,
`deltaRotation` ve paddle state'ini korudu; motion sıfır kaldı. Bu sonuç
vanilla server'ın remote boat fixture'ını ve bağlı entity akışını kabul eden
wire tüketimini gösterir; Java istemcisinin tam local/remote prediction ve
frame parity'sini göstermez. Vanilla server'ın `Entity.addPassenger` dalı
Player passenger'ı ilk sıraya taşıdığı için local Player'ın ikinci sırada
olduğu malformed client state canlı sunucudan üretilemedi; bu kaynak dalı
üstteki synthetic Rust testiyle ayrıca korunur.

## Açık kapsam

Nested/indirect local Player authority'si, boat status/friction ve buoyancy, collision-aware
`Entity.move`, ortak boat effects/animation tick'i, `ServerboundPaddleBoat` ve
`ServerboundMoveVehicle` paketleri, diğer vehicle türleri ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır.
