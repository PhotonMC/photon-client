# P08.ay.1 — ClientWorld remote Player vehicle attachment — [client-parity]

Bu kayıt, oak boat passenger'ı olan remote Player'ın Java istemcisindeki
`positionRider` sonucunun Rust `ClientWorld` kopyasında eşleşmesini karşılaştırır.
Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
production Rust adapter'a verilir.

[P08.ay server-conformance kaydı](client-world-entity-passenger-player-conformance.md)
vanilla sunucusuna bağlı Player attachment gözlemlerini tutar. O kayıt
server-conformance kanıtıdır; bu kayıt Java istemci state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `client/multiplayer/ClientLevel.java:461-491`, kök oak boat'ı bir kez
  tick'ler ve Player passenger'ını `Player` türü nedeniyle passenger dalında
  işler. Passenger için old transform ve tick count önce alınır.
- `world/entity/Entity.java:1812-1825,2380-2405`, `rideTick` hareketi
  sıfırlar, entity tick'ini yürütür ve bağlı aracın `positionRider` yolunu
  çağırır. Boat'ın riding position'ından passenger vehicle attachment noktası
  çıkarılır.
- `world/entity/Avatar.java:17-23` standing Player için attachment'ı
  `(0,0.6,0)` olarak tanımlar. `EntityTypes.java:1163-1172` bu attachment'ı
  Player entity dimensions'ına bağlar; snowball'ın feet fallback'inden
  farklı olarak y bileşeni sıfır değildir.
- `world/entity/vehicle/boat/AbstractBoat.java:132-150,631-684` oak boat'ın
  tek passenger ride height'ını `0.1875` yapar ve position sonrasında boat
  passenger rotation clamp yan etkisini yürütür. Bu fixture'da boat
  `deltaRotation=0` bırakılmıştır.

## Yeni dal ve varyant kararı

Bu kayıt, P08.ay server-conformance kaydındaki Player
`getVehicleAttachmentPoint` dalını ilk kez Java istemcisiyle
`client-parity` olarak çalıştırır. P08.ax.1 aynı boat koltuğunun snowball feet
fallback'ini karşılaştırdı; buradaki farklı gözlem Player dimensions'ından
gelen `0.6` y attachment'ının seat konumundan çıkarılmasıdır. Mevcut aynı
sonucu veren `client-parity` kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı normal oak boat ve remote Player için passenger
tick sırasını, movement reset'ini, previous position history'sini ve Player
attachment y sonucunu kapsar. Sabit değiştiren ayrı boat veya yaw kayıtları
açılmadı; boat seat ve rotation işlemleri P08.ax.1 ile ortak korunur.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, remote Player type ID `156`, boat yaw
  `90.0F` ve Player yaw/pitch `10.0F/-15.0F` kullanılır.
- Boat `(2,2,2)` konumunda, Player `(2,12,2)` konumunda ve başlangıç
  movement'i `(0.125,0,0)` ile başlar. Player tek ordered passenger'dır.
- World tick'inde boat kök olarak önce işlenir. Player passenger dalında
  movement sıfırlanır, tick count bir artırılır ve ardından oak boat'ın seat
  konumu uygulanır.
- Tek passenger seat'i `(2,2.1875,2)` olur. Player attachment `(0,0.6,0)`
  çıkarıldıktan sonra current position `(2,1.5875,2)`'dir.
- Player'ın previous position'ı tick başındaki `(2,12,2)` değeridir; seat
  taşıması current position'a yazılır. Passenger movement'i
  `(0,0,0)` olarak kalır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-player-position` kaydı initial ve
`player-seat` state'lerini aynı fixture üzerinde yürütür. Java oracle iki
çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`66aacfff53d9e67cc22f0438e3aa4ab4379ad5e4c81bbb738487cc1d319d8bf6`

Bu capture math ailesinde **3008** case içerir. Seçili Java state'i:

| State | Player position | Previous position | Motion |
| --- | --- | --- | --- |
| `player-seat` | `(2,1.5875,2)` | `(2,12,2)` | `(0,0,0)` |

Player rotation raw state'i `rotation_y=41200000` ve
`rotation_x=c1700000` olarak korunur; boat rotation clamp'i bu girdide
başlangıç yaw'ını değiştirmez.

Tam corpus Java capture kimliği:
`e8ffa80a292f26c3b9da6d4298a54db22df860cf7b7891efb38b408b570a0deb`

Toplam **3869** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_passengers` ve
`ClientWorld::tick` yollarını kullanır. Böylece Player entity'si için
production remote entity tick eligibility, movement reset, seat position,
attachment y hesabı ve rotation state'i karşılaştırılır.

Rust tarafında yeni bir sabit veya Player'a özel fixture sonucu eklenmedi;
mevcut `position_entity_passenger` yolu type ID `156` için
`VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y=0.6` değerini kullandı. Java oracle,
Player dimensions attachment'ını ve gerçek `Boat.positionRider` çağrısını
fixture üzerinde yürüttü. Bu karşılaştırma, beklenen seat y değerini Rust
çıktısından üretmeden kuruldu.

Debug adapter SHA-256:
`c9c7f551115557fe8c6c700717114135f8b361badf689bf49e157fc25b039fb0`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3008
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3869
difference=null
status=parity_passed
```

Release adapter SHA-256:
`cf68420a9d29850e32b71eb06f23ff78e494f7ab14212240cab05533b3fb26cd`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3869
difference=null
status=parity_passed
```

`cargo test --locked --offline` 194 test ile, clippy ve runner kontrolleri bu
commit öncesinde geçti. Vanilla server koşusu [P08.ay
server-conformance kaydında](client-world-entity-passenger-player-conformance.md)
tutulur.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun Player passenger
packet'lerini kabul etmesi [P08.ay server-conformance kaydında](client-world-entity-passenger-player-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Local `ClientPlayerState` için aynı attachment yolunun Java istemci state'iyle
ayrı karşılaştırması, crouching/swimming pose dimensions'ı, chest boat/raft,
nonzero boat `deltaRotation`, passenger rotation yan etkilerinin ayrı
dalları, nested controlling passenger, collision, dismount, gerçek
oynanabilir istemci bağlantısı ve render/frame timing'i açık kalır. Bu kayıt
P08'in `client-parity` kapanışı değildir.
