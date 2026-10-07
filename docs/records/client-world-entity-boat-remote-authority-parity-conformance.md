# P08.bk.1 — ClientWorld remote boat authority — [client-parity]

Bu kayıt, local Player tarafından kontrol edilmeyen oak boat entity'sinin
`ClientWorld` tick dalını Java 26.2 ile Rust world kopyasında karşılaştırır.
Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
production Rust adapter'a verilir.

[P08.bk server-conformance kaydı](client-world-entity-boat-remote-authority-conformance.md)
vanilla 26.2 dedicated server'dan gelen remote boat ve Zombie passenger wire
state'inin Rust world'ünde kabulünü gösterir. O kayıt canlı sunucu wire
kanıtıdır; bu kayıt Java istemci tick state'ini aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/vehicle/boat/AbstractBoat.java:210-251`, ortak
  `super.tick()` sonrasında local instance authority yoksa boat'ın
  `deltaMovement` değerini `Vec3.ZERO` yapar. `floatBoat`, client-side
  `controlBoat` ve `move` adımları yalnız authority dalında çalışır.
- `world/entity/vehicle/boat/AbstractBoat.java:760-768`, boat'ın controlling
  passenger'ını ilk living passenger'dan seçer. `world/entity/Entity.java:3508-3522,3587-3603`
  ve `world/entity/player/Player.java:1276-1283`, bu passenger'ın local Player
  olup olmamasından client authority sonucunun türediğini gösterir.
- `client/multiplayer/ClientLevel.java:471-490`, root vehicle'ın tick'ini
  passenger tick'inden önce yürütür ve önceki transform ile `tickCount`
  state'ini günceller. Remote boat branch'i input, buoyancy ve movement
  prediction uygulamaz.

Oracle, bu source-order davranışını açık bir remote fixture ile çalıştırır:
ilk passenger Zombie'dir, local Player world state'ine bağlanmaz ve input
bilerek sağ+ileri olarak verilir. Böylece authority kapısının input'tan önce
çalıştığı ölçülür; passenger'ın kendi tick ayrıntıları bu boat state kaydının
ötesindedir.

## Yeni dal ve varyant kararı

Bu dilim, P08.bk server-conformance kaydındaki remote/non-authoritative boat
dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; aynı root
tick içinde önceki position/rotation ve `tickCount` güncellenirken motion'ın
tam sıfırlanması, position/rotation/delta rotation/paddle state'lerinin ve
verilen input'ın etkisiz kalmasıdır. Aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; P08.bj.1 local-authority boat
dalını ve önceki generic projectile tick kayıtları farklı Java gövdelerini
ölçer.

İlk ve ikinci remote tick aynı tek parametrik corpus kaydında ardışık
işlemlerle kapsandı. Yalnız sabiti değiştiren veya aynı branch'i kopyalayan
yeni boat varyantı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`, oak Boat entity type
  ID `89` ve Zombie entity type ID `151`'dir. Başlangıç world'ü dimension min Y
  `0` olan, chunk `(0,0)` içinde boş ve yüklü tek section'dır.
- Boat entity `37`, `(2,12,2)` konumunda `(0.25,0.5,-0.75)` movement'i ve
  `(y=10,x=-20)` rotation'ı ile başlar. Zombie entity `38`, boat'ın ilk ve
  tek passenger'ıdır. Local Player ID bağlanmaz; bu nedenle boat remote
  authority dalındadır.
- Sağ+ileri input boat'a uygulanmış görünür ancak controlling passenger local
  Player olmadığı için `floatBoat`, `controlBoat` ve `move` çalışmaz. Her tick
  sonunda motion `(0,0,0)` olur; position, rotation, delta rotation ve paddle
  state'i başlangıç değerinde kalır.
- ClientLevel tick'i önceki position/rotation state'ini ve `tickCount`'ı yine
  günceller. İki ardışık tick, bu geçmiş state'inin de Java ile eşleştiğini
  karşılaştırır.
- Nested veya indirect controlling passenger, server-side passenger reorder,
  boat status/friction ve buoyancy, collision-aware `Entity.move`, passenger
  `rideTick` ayrıntıları, serverbound paddle/movement paketleri, diğer vehicle
  türleri, renderer ve oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-boat-remote-tick` kaydı bir remote oak boat, Zombie
passenger, etkisiz sağ+ileri input ve iki tick state geçişi taşır. Java oracle
iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Java referans kimliği:
`df1269d21024a49e670fc788c36dbef157339734fd3771eab3cf0d56e2779594`

Math corpus'u **2998** kayıttır. Remote boat'ın seçili raw state bitleri:

| Adım | position X/Y/Z | motion X/Y/Z | rotation Y/X | previous rotation Y/X | delta rotation | paddle | tick |
| --- | --- | --- | --- | --- | --- | --- | ---: |
| initial | `4000000000000000/4028000000000000/4000000000000000` | `3fd0000000000000/3fe0000000000000/bfe8000000000000` | `41200000/c1a00000` | `41200000/c1a00000` | `00000000` | `false/false` | 0 |
| remote-first-tick | `4000000000000000/4028000000000000/4000000000000000` | `0000000000000000/0000000000000000/0000000000000000` | `41200000/c1a00000` | `41200000/c1a00000` | `00000000` | `false/false` | 1 |
| remote-second-tick | `4000000000000000/4028000000000000/4000000000000000` | `0000000000000000/0000000000000000/0000000000000000` | `41200000/c1a00000` | `41200000/c1a00000` | `00000000` | `false/false` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Adapter boş bir
section, oak boat, Zombie passenger, passenger ilişkisi ve boat input'ı kurar;
local Player state'i kurmadığı için production remote authority dalını
çalıştırır. State serializer position, previous position, motion, rotation
geçmişi, `tickCount`, no-gravity, delta rotation ve paddle alanlarını okur.

Debug adapter SHA-256:
`b5f81b631f9fea9880ac679f9fee2cb159057d609416eb6674d28fc6f293d1d1`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2998
difference=null
status=parity_passed
```

Release adapter SHA-256:
`fd30228fcb8d01d6635a7a9032e0acaad2b67bb801c5ccf5f1f0cf583e8088f5`

Release adapter aynı capture üzerinde **2998/2998** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2998
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
remote boat AddEntity ve passenger kabul sonucu [P08.bk
server-conformance kaydında](client-world-entity-boat-remote-authority-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Nested/indirect local Player authority'si, server-side passenger reorder,
boat status/friction ve buoyancy, fluid/shape sorgusu, collision-aware
movement, passenger tick/attachment/rotation yan etkileri, serverbound paddle
ve vehicle movement paketleri, diğer vehicle türleri, genel motion/entity
lifecycle, renderer ve oynanabilir istemci bağlantısı açık kalır.
