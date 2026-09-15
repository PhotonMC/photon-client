# P08.bj.1 — ClientWorld local boat `IN_AIR` tick — [client-parity]

Bu kayıt, local Player tarafından kontrol edilen oak boat entity'sinin yüklü ve
boş bir world section içindeki `IN_AIR` tick'ini Java 26.2 ile Rust
`ClientWorld` kopyasında karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı
özgün sentetik input Java oracle'ına ve production Rust adapter'a verilir.

[P08.bj server-conformance kaydı](client-world-entity-boat-air-conformance.md)
vanilla 26.2 dedicated server'dan gelen local passenger/boat wire state'inin
Rust world tick'inde kabulünü gösterir. O kayıt canlı sunucu wire kanıtıdır; bu
kayıt Java istemci tick state'ini aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/vehicle/boat/AbstractBoat.java:210-251`, local instance
  authoritative olduğunda boat tick'inin `floatBoat`, client-side
  `controlBoat` ve `move` sırasını yürütür. `client/multiplayer/ClientLevel.java:471-490`,
  root vehicle'ı passenger'dan önce tick eder; `Entity.java:3587-3603` ve
  `LocalPlayer.java:975-978`, bu dalın local controlling Player ve input
  bayraklarına bağlı olduğunu gösterir.
- `AbstractBoat.java:538-580`, `IN_AIR` durumunda x/z movement ve
  `deltaRotation` için f32 `0.9F` friction, gravity açıkken y movement'inden
  `0.04` çıkarılmasını ve `NoGravity` dalında bunun atlanmasını seçer.
- `AbstractBoat.java:586-620`, sağ input ile `deltaRotation`'ı artırır, güncel
  yaw'a ekler, ileri input için f32 `0.04F` ivmeyi Java `Mth.sin`/`Mth.cos`
  yönüne taşır ve iki paddle state'ini günceller. `Entity.java:712-802`, bu
  movement ile son konum adımını yürütür.

Boat status, su/kara friction'ı ve collision shape çözümü bu kayıt dışında
kalacak şekilde Java fixture açık hava `IN_AIR` dalının source-order aritmetiğini
çalıştırır. Trigonometrik sonuçlar pinned JAR'daki `Mth.sin`/`Mth.cos`
tablolarından alınır; input her adımda local controlling Player input'ı olarak
verilir.

## Yeni dal ve varyant kararı

Bu dilim, P08.bj server-conformance kaydındaki local-authority `IN_AIR` boat
dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; aynı tick
içinde `0.9F` x/z friction ve gravity adımından sonra input kaynaklı yaw,
forward acceleration, paddle state ve position güncellemesinin Java ile aynı
sırada üretilmesidir. Aynı sonucu veren mevcut `client-parity` kayıtlarının
sayısı **0**'dır; önceki entity tick kayıtları Snowball, Arrow, Experience Orb,
Primed TNT, FallingBlock ve Firework Rocket sınıflarının farklı tick gövdelerini
ölçer.

Gravity açık satır ve `NoGravity` satırı tek parametrik corpus kaydında aynı
tekne üzerinde ardışık işlemlerle kapsandı. Yalnız sabiti değiştiren yeni boat
varyantı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve oak Boat entity type
  ID `89`'dur. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)`
  içinde boş ve yüklü tek section'dır.
- Boat entity `37`, `(2,10,2)` konumunda `(0.25,0.5,-0.75)` movement'i ve
  `(y=10,x=-20)` rotation'ı ile başlar. Player entity `38` ilk passenger ve
  local controlling Player'dır. Input `right=true`, `forward=true`, diğer
  üç input bayrağı `false`'dur.
- İlk tick'te `IN_AIR` float adımı x/z movement'i `0.9F` ile çarpar, gravity
  açık olduğu için y movement'inden `0.04` çıkarır ve `deltaRotation`'ı `0.9F`
  ile çarpar. Sonra control adımı sağ dönüşü ve ileri `0.04F` ivmesini uygular;
  paddle state'i `(true,true)` olur ve boat bu movement ile konum alır.
- İlk tick sonrasında yalnız entity no-gravity metadata'sı `true` yapılır.
  İkinci tick aynı input'u sürdürür; x/z friction ve control devam eder,
  gravity y adımı atlanır. Her world tick'inde önceki position/rotation ve
  `tickCount` state'i de karşılaştırılır.
- Su ve kara status/friction'ı, buoyancy, fluid, collision-aware `move`,
  world shape sorgusu, passenger seat/rotation yan etkileri, serverbound paddle
  ve vehicle movement paketleri, nested passenger, paddle animasyonu/sesi,
  gerçek pencere/input callback'i, renderer ve oynanabilir istemci bu kaydın
  dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-boat-air-tick` kaydı bir oak boat, bir local passenger,
üç input/tick state geçişi ve bir no-gravity metadata geçişi taşır. Java oracle
iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Java referans kimliği:
`af981fb015f51c70c6ada4da60d8c84820dec5c50ef6092c6325356d29bfe69f`

Math corpus'u **2997** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | position X/Y/Z | motion X/Y/Z | rotation Y/X | delta rotation | paddle | tick |
| --- | --- | --- | --- | --- | --- | ---: |
| initial | `4000000000000000/4024000000000000/4000000000000000` | `3fd0000000000000/3fe0000000000000/bfe8000000000000` | `41200000/c1a00000` | `00000000` | `false/false` | 0 |
| air-gravity-control | `4001bd2c35100000/4024eb851eb851ec/3ff5d40805000000` | `3fcbd2c351000000/3fdd70a3d70a3d71/bfe457eff6000000` | `41300000/c1a00000` | `3f800000` | `true/true` | 1 |
| enable-no-gravity | `4001bd2c35100000/4024eb851eb851ec/3ff5d40805000000` | `3fcbd2c351000000/3fdd70a3d70a3d71/bfe457eff6000000` | `41300000/c1a00000` | `3f800000` | `true/true` | 1 |
| air-no-gravity-control | `40033b8ad4192184/4025d70a3d70a3d8/3fea9854de565ffc` | `3fc7e5e9f0921846/3fdd70a3d70a3d71/bfe10fbb2ba9a004` | `414e6666/c1a00000` | `3ff33333` | `true/true` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Adapter açık bir
section, oak boat, local Player, passenger ilişkisi ve `BoatInput` kurar;
`ClientEntity::tick_boat_float_in_air`, `tick_boat_control` ve position adımı
aynı world tick içinde yürütülür. No-gravity geçişi gerçek entity metadata
uygulamasıyla yapılır.

Debug adapter SHA-256:
`3e2e1ab0829e83629660d7aff47320a3818e3a2908f33eccdd5314b2c6f5a6ac`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2997
difference=null
status=parity_passed
```

Release adapter doğrulaması aynı capture üzerinde çalıştırıldı ve **2997/2997**
kayıt `difference=null` ile geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java oracle'ına ve Rust
adapter'a verilir. Vanilla sunucusu üzerindeki local boat AddEntity ve
passenger kabul sonucu [P08.bj server-conformance kaydında](client-world-entity-boat-air-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Su/kara friction ve buoyancy, fluid/shape sorgusu, collision-aware movement,
boat status geçişleri, passenger attachment/rotation, nested controlling
passenger, paddle animasyonu/sesi, serverbound paddle ve vehicle movement
paketleri, gerçek input/frame zamanlaması, renderer, genel motion/entity
lifecycle ve oynanabilir istemci bağlantısı açık kalır.
