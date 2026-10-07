# P08.bm.1 — ClientWorld Experience Orb tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki Experience Orb entity'sinin
ilk iki `ClientWorld` tick'inde Java 26.2 ile Rust world kopyasında aynı
konum, hareket, rotation ve tick state'ini üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.bm server-conformance kaydı](client-world-entity-experience-orb-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan Experience Orb AddEntity state'inin
Rust world tick'inde ilerletildiğini gösterir. O kayıt canlı sunucu wire ve
kabul kanıtıdır; bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/EntityTypes.java:440-443`, Experience Orb entity type ID'sini
  `49` olarak kaydeder. `client/multiplayer/ClientLevel.java:467-472`, uygun
  entity için eski transform'u kopyalar, `tickCount` değerini artırır ve
  entity tick gövdesini yürütür.
- `world/entity/ExperienceOrb.java:102-152`, client tarafındaki ilk tick'te
  `firstTick` state'ini kapatıp entity gövdesini atlar. Sonraki tick'te
  çarpışmasız ve fluidsiz dalda gravity uygular, hareket eder ve position
  adımından sonra air drag uygular.
- `world/entity/ExperienceOrb.java:154-156`, Experience Orb air drag'ını
  `0.98F`; aynı sınıfın `getDefaultGravity` gövdesi (`:96-99`) gravity'yi
  `0.03` olarak tanımlar. `world/entity/Entity.java:1510-1529`, `NoGravity`
  true olduğunda gravity'nin sıfırlanmasını belirler.
- `world/entity/ExperienceOrb.java:113-132`, oyuncuya yönelme, lava ve merge
  dallarını da içerir. Bu kayıt, oyuncu bulunmayan boş hava hücresindeki
  gravity/position/friction yolunu izole eder.

Snowball ve Arrow free-flight kayıtlarından farklı Java dalı, client'a yeni
gelen Experience Orb'un ilk tick'te yalnız `firstTick` state'ini kapatmasıdır.
Sonraki tick'te hareket sırası da Experience Orb'un kendi `gravity → position
→ air drag` gövdesinden gelir. Oracle, `ExperienceOrb` tabanlı özgün fixture
ile gerekli inherited gravity ve air drag çağrılarını aynı sırada yürütür;
level collision, fluid, player ve merge yan etkilerini kontrollü olarak dışarıda
bırakır.

## Yeni dal ve varyant kararı

Bu dilim, P08.bm server-conformance kaydındaki Experience Orb client tick
dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; ilk client
tick'inde position ve movement'in korunması, ikinci tick'te `0.03` gravity'nin
position adımından önce ve `0.98F` drag'ın sonra uygulanmasıdır. Aynı sonucu
veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; Snowball ve Arrow
kayıtları ayrı projectile tick gövdelerini ölçer.

Normal gravity ve `NoGravity` girdileri, ilk tick korumasını ve sonraki tick
sırasını tek parametrik corpus kaydının iki entity satırı olarak kapsar. Aynı
Java dalının yalnız sabiti değişen kopyası açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Experience Orb entity
  type ID `49`'dur. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)`
  içinde boş ve yüklü tek section'dır.
- Entity `37`, `(2,10,2)` konumunda `(0.125,0.25,-0.125)` hareketi ve
  `(y=10,x=20)` rotation'ı ile; entity `38`, `(4,10,2)` konumunda
  `(0,0,1)` hareketi ve `(y=45,x=30)` rotation'ı ile başlar. Entity `37`
  gravity kullanır, entity `38` `NoGravity` kullanır.
- İlk world tick'inde ClientLevel eski position/rotation state'ini saklar,
  `tickCount` değerini `1` artırır, Experience Orb `firstTick` korumasıyla
  position ve movement'i değiştirmez. İkinci tick'te eski state saklanır ve
  `tickCount` `2` olur.
- İkinci tick'te `NoGravity=false` satırında önce movement Y'den `0.03`
  çıkarılır; entity gravity uygulanmış movement ile konum alır, sonra üç
  movement bileşeni Java f32 `0.98F` değerinin f64 işlemine genişletilmiş
  karşılığıyla çarpılır. `NoGravity=true` satırında gravity çıkarılmaz.
- Bu dar dalda rotation değişmez. Yakındaki oyuncuya yönelme, su/lava
  hareketi, block/entity collision, ground friction, vertical bounce, merge,
  age/lifetime, pickup/effect, removal, gerçek packet/thread dispatch'i,
  renderer ve oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-experience-orb-tick` kaydı iki Experience Orb satırını ve
iki tick işlemini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar
farkı `null`'dır.

Java referans kimliği:
`a885d67e09b67d1a27d272a12a776d9f87bce36bec81369be2814df5085e661a`

Math corpus'u **2993** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | Entity | position Y/Z | motion Y/Z | rotation Y | rotation X | tick |
| --- | ---: | --- | --- | --- | --- | ---: |
| initial | 37 | `y=4024000000000000` | `y=3fd0000000000000` | `41200000` | `41a00000` | 0 |
| first-client-tick | 37 | `y=4024000000000000` | `y=3fd0000000000000` | `41200000` | `41a00000` | 1 |
| second-client-tick | 37 | `y=402470a3d70a3d71` | `y=3fcb98c7eb851eb8` | `41200000` | `41a00000` | 2 |
| initial | 38 | `z=4000000000000000` | `z=3ff0000000000000` | `42340000` | `41f00000` | 0 |
| first-client-tick | 38 | `z=4000000000000000` | `z=3ff0000000000000` | `42340000` | `41f00000` | 1 |
| second-client-tick | 38 | `z=4008000000000000` | `z=3fef5c2900000000` | `42340000` | `41f00000` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Yüklü air-cell
kontrolü type ID `49` için geçildiğinde `ClientEntity` ilk tick'te
`first_tick` state'ini kapatır; sonraki tick'te
`tick_experience_orb`, gravity/position/air friction sırasını uygular.
Önceki position ve tick count state'leri aynı world tick preamble'ından
sorgulanır.

Debug adapter SHA-256:
`0b3e3227e9adb5f0cb1f0944660436b7dd3e554cf6c589a54f58a7ed1701354b`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2993
difference=null
status=parity_passed
```

Release adapter SHA-256:
`5a98d3b85d0ca2998cc113f3f99e8ff52d138fc7ca064a9171a6015fa1d1b11e`

Release adapter aynı capture üzerinde **2993/2993** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2993
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
Experience Orb AddEntity ve world tick sonucu [P08.bm
server-conformance kaydında](client-world-entity-experience-orb-tick-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Experience Orb'un oyuncuya yönelme ve takip state'i, su/lava hareketi,
block/entity collision, ground friction/bounce, merge, age/lifetime,
pickup/effect, removal, network entity dispatch, renderer, genel motion
integration ve oynanabilir istemci bağlantısı açık kalır.
