# P08.aw.1 — ClientWorld passenger tick and `rideTick` — [client-parity]

Bu kayıt, araç altındaki entity'nin Java istemcisindeki tick sırasının ve
`rideTick` hareket sıfırlamasının Rust `ClientWorld` kopyasında eşleşmesini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java
oracle'ına ve production Rust adapter'a verilir.

[P08.aw server-conformance kaydı](client-world-entity-passenger-tick-conformance.md)
vanilla sunucusuna bağlı world tick gözlemini tutar. O kayıt server-conformance
kanıtıdır; bu kayıt Java `ClientLevel` tick state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientLevel.java:461-491`, üst seviye ticking entity
  listesinden passenger olmayan kökleri seçer. Kök için old transform ve tick
  count güncellenir; yolcular aynı araç ilişkisi korunuyorsa recursive olarak
  `tickPassenger` dalına gider.
- Aynı `ClientLevel.java:479-490` dalı yalnız Player veya ticking entity
  passenger'ını işler. Passenger üst seviye listede ayrıca yürütülmez ve
  `rideTick` çağrısından önce ikinci bir kök tick'i almaz.
- `world/entity/Entity.java:1812-1825,2380-2390`, tick başlangıcındaki history
  snapshot'ını, `rideTick` içinde movement'in `Vec3.ZERO` yapılmasını, entity
  tick'ini ve sonrasındaki rider konumlandırmasını belirler.
- `world/entity/projectile/ThrowableProjectile.java:46-75`, snowball'ın
  gravity/inertia, hareket, rotation ve air-drag sırasını belirler.

Oracle, kök ve passenger sırasını kaynak eşdeğeri fixture akışıyla yürütür.
Snowball hareket adımı Java'nın gerçek `applyGravity`, `getAirDrag` ve
`updateRotation` çağrılarını kullanır; profiler, collision hit ve
`positionRider` yan etkileri bu dar state karşılaştırmasına dahil edilmez.

## Yeni dal ve varyant kararı

Bu kayıt, P08.aw server-conformance passenger tick dalını ilk kez
`client-parity` sınıfında çalıştırır. P08.av.1 yalnız ilişki graph'ının packet
replacement state'ini karşılaştırır; P08.aq.1 ise snowball'ın kök free-flight
tick'ini ölçer. Buradaki farklı gözlem passenger'ın root tick listesinden
çıkarılması, recursive olarak bir kez işlenmesi, `rideTick` başında mevcut
movement'inin sıfırlanması ve aynı world tick'inde bağımsız kök snowball'ın
normal hareketini korumasıdır. Aynı sonucu veren mevcut client-parity
kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı bir generic vehicle, bir bağlı snowball ve bir kök
snowball ile iki ardışık world tick'ini kapsar. Sabit değiştirerek aynı tick
dalını çoğaltan ayrı bir kayıt açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Boş bir loaded
  section içinde vehicle type ID `71`, passenger ve kök projectile type ID
  `120` kullanılır.
- Vehicle `(2,2,2)` konumundadır. Passenger aynı konumda başlangıç motion'ı
  `(0.125,0,0)` ile araca bağlıdır. Kök snowball `(5,2,2)` konumunda aynı
  motion'la bağımsızdır.
- Her world tick'inde vehicle önce bir kez tick edilir. Bağlı passenger root
  listesinden atlanır; ilişkisi doğrulanır, tick count'u bir artırılır ve
  `rideTick` movement'i sıfırladıktan sonra snowball tick'i gravity ve air drag
  uygular. Passenger'ın yatay hareketi bu nedenle sıfır kalır.
- Bağımsız kök snowball aynı tick'te sıfırlama olmadan normal free-flight
  motion'ını yürütür. Vehicle'ın generic tick gövdesi bu fixture'da state'ini
  değiştirmez, ancak tick count/history snapshot'ı alınır.
- Çıktı her entity için tick count, position, previous position, motion,
  rotation ve no-gravity state'ini taşır. `positionRider`, boat control,
  collision, removal/freeze gate'i, nested passenger ve gerçek render/frame
  timing'i bu dilimin dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-tick` kaydı birinci ve ikinci tick state'ini
aynı bağlı/kök karşılaştırması üzerinde yürütür. Java oracle iki çalıştırmada
kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`2d76b876b3623ba5f39883fdc66632f1bdc0e63768baaff28ffb30d32e67ef83`

Bu capture math ailesinde **3006** case içerir. Seçili davranış:

| Entity | İlk tick sonucu | İkinci tick sonucu |
| --- | --- | --- |
| vehicle `37` | `tick_count=1`, state position/motion korunur | `tick_count=2`, state position/motion korunur |
| bağlı passenger `38` | `position.y=1.9703`, motion `(0,-0.0297,0)`; x hareketi yok | ikinci kez aynı `rideTick` reset ve gravity sırası |
| kök snowball `39` | x/z başlangıç hareketiyle ilerler, motion sıfırlanmaz | normal free-flight state'i devam eder |

Tam corpus Java capture kimliği:
`98229e3e8b3b6ba2d31843a6fee344c3530b025c63a27d6dfc8bd5942fbbe09c`

Toplam **3867** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_passengers` ve
`ClientWorld::tick` yollarını kullanır. Bu nedenle passenger'ın `vehicle_id`
ilişkisi root entity filtresine, motion reset'i mevcut passenger tick yoluna ve
snowball gravity/air-drag state'i production entity tick gövdesine bağlanır.

Debug adapter SHA-256:
`36968bf6b2c690cc6a18f770857784ec0acc481087cb4e1deb5494deb2d`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3006
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3867
difference=null
status=parity_passed
```

Release adapter SHA-256:
`146a15b43cba1637372d8094005bee6ce927b773a6e35716156e5166d5112a13`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3867
difference=null
status=parity_passed
```

İlgili Rust unit testleri, clippy ve runner kontrolleri bu commit öncesinde
ayrıca çalıştırıldı. Tam vanilla server koşusu [P08.aw
server-conformance kaydında](client-world-entity-passenger-tick-conformance.md)
tutulur.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun passenger packet ve
world tick akışını kabul etmesi [P08.aw server-conformance kaydında](client-world-entity-passenger-tick-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Passenger `positionRider` geometrisi, nested passenger recursive state'i,
vehicle control/physics, collision-aware movement, removal/freeze koşulları,
boarding cooldown, passenger transform history'nin tam packet akışı, gerçek
oynanabilir istemci bağlantısı ve render/frame timing'i açık kalır. Bu kayıt
P08'in `client-parity` kapanışı değildir.
