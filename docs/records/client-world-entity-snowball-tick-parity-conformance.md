# P08.aq.1 — ClientWorld Snowball free-flight tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki vanilla Snowball entity'sinin
iki ardışık `ClientWorld` tick'inde Java 26.2 ile Rust world kopyasında aynı
konum, hareket, rotation ve tick state'ini üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.aq server-conformance kaydı](client-world-entity-snowball-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan AddEntity movement state'inin Rust
world tick'inde ilerletildiğini gösterir. O kayıt canlı sunucu wire ve kabul
kanıtıdır; bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `network/protocol/game/ClientboundAddEntityPacket.java:17-31,95-120` ve
  `client/multiplayer/ClientPacketListener.java:565-590`, Snowball type ID
  `120` için başlangıç konumu, rotation ve hareket state'inin ClientWorld
  entity'sine bağlanmasını tanımlar.
- `client/multiplayer/ClientLevel.java:450-477`, tick'e uygun entity için eski
  transform'u kopyalar, `tickCount` değerini artırır ve entity tick gövdesini
  çalıştırır.
- `world/entity/projectile/ThrowableProjectile.java:46-65,67-97,110-113`,
  gravity → air inertia → MISS konum sırasını, Snowball'ın `0.99F` drag'ını
  ve `0.03` varsayılan gravity'sini tanımlar.
- `world/entity/projectile/Projectile.java:329-346`, post-drag hareketten
  pitch/yaw hedefi üretir, `lerpRotation` ile eski rotation'a `0.2F` yaklaşır
  ve 180° sınırında wrapped farkı korur. Pinned client jar bytecode'unda bu
  dönüşüm `57.2957763671875` derece/radyan çarpanı olarak gerçekleşir; Rust
  karşılığı aynı sabiti kullanır.
- `world/entity/Entity.java:1510-1529`, `NoGravity` state'inin gravity'yi
  sıfırladığını; `world/phys/Vec3.java:188-190` ve
  `util/Mth.java:391-438,550-552`, yatay uzaklık, `atan2` ve f32 lerp işlem
  sırasını doğrular.

Oracle, `Snowball` tabanlı özgün fixture ile ClientLevel preamble'ındaki eski
transform/tick sayacı adımlarını ve ThrowableProjectile'ın yüklü air-cell,
MISS free-flight dalını aynı sırada yürütür. Fixture, collision, bubble column,
block effect ve gerçek level yan etkilerini bu state karşılaştırması için
kontrollü olarak dışarıda bırakır.

## Yeni dal ve varyant kararı

Bu dilim, P08.aq server-conformance kaydındaki Java ThrowableProjectile
free-flight dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem;
Snowball type ID `120` için iki ardışık world tick'inde gravity/no-gravity
ayrımıyla birlikte previous position, post-drag movement, rotation ve
`tickCount` state'inin Java ile aynı raw bitlerde ilerlemesidir. Aynı sonucu
veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; önceki P08 parity
kayıtları metadata, interpolation veya başka entity state dallarını ölçer.

Aynı kod yolunun sabit değişen kopyalarını çoğaltmamak için normal gravity ve
`NoGravity` girdileri tek parametrik corpus kaydının iki entity satırı olarak
verildi; ikinci satırın `355° → 0°` yaw geçişi `Projectile.lerpRotation`
wrapped dalını da çalıştırır. İki ardışık tick tek kayıtta kapsandı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Snowball entity type
  ID `120`'dir. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)`
  içinde boş ve yüklü tek section'dır.
- Entity `37`, `(2,1,2)` konumunda `(0.25,0.5,0.25)` hareketi ve
  `(y=10,x=-20)` rotation'ı ile; entity `38`, `(4,1,2)` konumunda
  `(0,0,1)` hareketi ve `(y=355,x=30)` rotation'ı ile başlar. Entity `37`
  gravity kullanır, entity `38` `NoGravity` kullanır.
- Her world tick'inde previous position/rotation önce current state'ten
  alınır ve `tickCount` bir artırılır. Normal Snowball hareketinde Y'ye
  `-0.03` eklenir; iki entity'nin üç hareket bileşeni Java `0.99F` air drag
  ile çarpılır. `NoGravity` satırında yalnız drag uygulanır.
- Yüklü air-cell ve MISS yolunda yeni position post-drag movement'ın mevcut
  position'a eklenmesidir. Aynı post-drag movement'tan Java `atan2` ve yatay
  uzaklık ile hedef pitch/yaw üretilir; eski rotation'a `0.2F` ile yaklaşılır.
- Block/entity collision, projectile impact/removal, bubble column, fluid,
  block effect, gerçek packet/thread dispatch'i, renderer ve oynanabilir
  istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-snowball-tick` kaydı iki Snowball satırını ve iki tick
işlemini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı
`null`'dır.

Java referans kimliği:
`4378144c710b591b1d8b34d3d63b92aa4e5c16546116e6e830b63ddf39a7172a`

Math corpus'u **2991** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | Entity | position bileşeni | motion bileşeni | rotation Y | rotation X | tick |
| --- | ---: | --- | --- | --- | --- | ---: |
| initial | 37 | `y=3ff0000000000000` | `y=3fe0000000000000` | `41200000` | `c1a00000` | 0 |
| first-air-tick | 37 | `y=3ff771de6ae147ae` | `y=3fddc779ab851eb8` | `41880000` | `c0ac7e30` | 1 |
| second-air-tick | 37 | `y=3ffe57071ba4723a` | `y=3fdb94a2c30caa32` | `41b4cccc` | `40bdabea` | 2 |
| initial | 38 | `z=4000000000000000` | `z=3ff0000000000000` | `43b18000` | `41f00000` | 0 |
| first-air-tick | 38 | `z=4007eb8520000000` | `z=3fefae1480000000` | `c0800000` | `41c00000` | 1 |
| second-air-tick | 38 | `z=400fc2c3cdbf4880` | `z=3fef5cfab6fd2200` | `c04ccccd` | `4199999a` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Yüklü air-cell
kontrolü type ID `120` için geçildiğinde `ClientEntity` eski transform'u ve
tick sayacını kaydeder; Snowball gravity/drag/position/rotation adımları aynı
Java işlem sırasıyla uygulanır. Java'nın derlenmiş derece/radyan sabitinden
kaynaklanan ULP farkı `57.2957763671875` sabitiyle giderildi.

Debug adapter SHA-256:
`b0d8be538c3ae4f9f0714e3501b7569d61cb53909d684d4d9b79681b254c2000`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2991
difference=null
status=parity_passed
```

Release adapter SHA-256:
`516d727384768f8e41e5721ee45b691c0ad4659e69bff33a79b93751a0994d8d`

Release adapter aynı capture üzerinde **2991/2991** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2991
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
Snowball AddEntity ve world tick sonucu [P08.aq server-conformance kaydında](client-world-entity-snowball-tick-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Projectile collision ve hit/impact sonucu, bubble-column ve fluid dalları,
block effect/super tick yan etkileri, projectile removal, diğer projectile ve
living entity tick gövdeleri, network entity dispatch, renderer, genel motion
integration ve oynanabilir istemci bağlantısı açık kalır.
