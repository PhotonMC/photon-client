# P08.at-player-motion — [server-conformance] Remote Player motion interpolation

Bu kayıt, vanilla sunucusunun remote Player için gönderdiği
`ClientboundSetEntityMotionPacket` sonrasında Java istemci state'inin motion
değerini üç world tick'ine yaymasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated server'a
bağlanır. Java istemcisiyle aynı girdide frame/state karşılaştırması
yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`client/multiplayer/ClientPacketListener.java:621-627`, mevcut entity için
`lerpMotion` çağırır. `world/entity/Entity.java:2644-2646`, genel entity yolunda
bu çağrıyı doğrudan `setDeltaMovement` olarak uygular.

`client/player/RemotePlayer.java:12,43-62,72-76`, remote Player'ın override
yolunu tanımlar: packet motion değeri ayrı hedef state'e yazılır ve kalan adım
sayısı entity update interval'ının bir fazlası olur. `EntityTypes.java:1163-1172`,
Player update interval'ını `2` olarak sabitler; bu nedenle hedef üç adımda
uygulanır. `RemotePlayer.aiStep` her adımda mevcut motion ile hedef arasındaki
farkı kalan adım sayısına böler, `addDeltaMovement` ile ekler ve sayacı azaltır.
Bu sıra position interpolation ve head interpolation'dan sonradır.

`server/level/ServerEntity.java:177-194`, living entity motion değiştiğinde
izleyen istemcilere aynı clientbound motion packet'inin gönderildiğini
gösterir. Bu canlı senaryoda hareket girdisi, vanilla TNT patlamasının remote
Player'a verdiği knockback'tir.

## Sözleşme

- AddEntity başlangıç motion'ı anında state'e yazılır; bu kayıt o ayrı yolu
  değiştirmez.
- Clientbound motion packet'i type ID `156` olan remote Player'a geldiğinde
  packet hedefi ayrı motion interpolation state'inde tutulur; packet anında
  mevcut motion hedefe sıçramaz.
- Player update interval `2` olduğundan kalan adım sayısı `3` olur.
- Her world tick'inde `target - current` farkının mevcut kalan adım sayısına
  bölümü mevcut motion'a eklenir; üçüncü adımda motion packet hedefiyle aynı
  olur.
- Player dışı entity'ler genel Java `Entity.lerpMotion` yolundaki ham motion
  replacement davranışını korur.

## Rust doğrulaması

`ClientEntity`, remote Player için motion hedefini ve kalan adım sayısını ayrı
bit state'inde tutar. `ClientWorld::tick`, position ve head interpolation'dan
sonra Player motion interpolation'ını yürütür. `apply_entity_motion`, Player
dışında eski ham replacement yolunu, Player'da ise Java'nın üç adımlı hedef
yolunu seçer.

`remote_player_motion_packet_interpolates_for_three_ticks` unit testi sıfır
motion'dan `(0.3,-0.6,0.9)` hedefinin üç world tick sonucunu ve packet anında
mevcut motion'ın korunmasını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer ve mover Rust probe'ları aynı vanilla oturumuna bağlandı. Mover'ın
koordinatına sunucu konsolundan bir tick fuse'lu TNT bırakıldı:

```text
cargo run --locked --offline --example world_remote_player_motion_probe -- 127.0.0.1:25568 P08MotionObs18 P08MotionMove18
# vanilla server console, while both probes are connected
summon minecraft:tnt 22.5 80 -34.5 {Fuse:1}
```

Patlama mover Player'a yalnızca Y yönünde `0.8172495879875481` motion verdi.
Observer probe'u ilk farklı remote Player motion packet'ini aldı ve aynı
uygulamadan sonra üç world tick yürüttü:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `mover_movement_packets_sent` | `1` |
| `remote_player_id` | `5051` |
| packet öncesi motion | `(0.0, 0.0, 0.0)` |
| packet hedefi | `(0.0, 0.8172495879875481, 0.0)` |
| packet uygulaması sonrası motion | `(0.0, 0.0, 0.0)` |
| tick 1 motion | `(0.0, 0.2724165293291827, 0.0)` |
| tick 2 motion | `(0.0, 0.5448330586583654, 0.0)` |
| tick 3 motion | `(0.0, 0.8172495879875481, 0.0)` |
| `entity_motion_packet_count` | `1980` |
| `world_entity_count` | `48` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `8304` |

Probe, gözlenen üç state'i Java `RemotePlayer` adım sırasındaki fark bölme
hesabıyla karşılaştırdı ve başarılı çıktı. Packet ve world state gözlemi
vanilla sunucusunun Rust bağlantısını kabul ettiğini gösterir; Java
istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Generic LivingEntity ve diğer entity türlerinin gerçek client tick/fizik,
yerçekimi, collision ve push davranışı; teleport packet'inin delta movement ve
relative delta bitleri; local Player authority, passenger ilişkisi, hızlı
ardışık motion packet'leri, render partial tick ve Java istemcisiyle
`client-parity` karşılaştırması açık kalır. AddEntity ham motion state'i
[P08.e motion kaydında](client-world-entity-motion-conformance.md), remote
Player position interpolation [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md)
ve uzaklık üstü snap [P08.as kaydında](client-world-entity-player-snap-conformance.md)
izlenir. Bu kayıt da P08'in `client-parity` kapanışı değildir.
