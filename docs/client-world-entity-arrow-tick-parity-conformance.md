# P08.bl.1 — ClientWorld Arrow free-flight tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki normal Arrow entity'sinin
iki ardışık `ClientWorld` tick'inde Java 26.2 ile Rust world kopyasında aynı
konum, hareket, rotation ve tick state'ini üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.bl server-conformance kaydı](client-world-entity-arrow-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan Arrow AddEntity state'inin Rust world
tick'inde ilerletildiğini gösterir. O kayıt canlı sunucu wire ve kabul
kanıtıdır; bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/EntityTypes.java:190-193`, Arrow entity type ID'sini `6` olarak
  kaydeder. `client/multiplayer/ClientLevel.java:450-477`, uygun entity için
  eski transform'u kopyalayıp `tickCount` değerini artırır ve tick gövdesini
  yürütür.
- `world/entity/projectile/arrow/AbstractArrow.java:174-275`, fizik açık,
  in-ground olmayan ve hava içindeki Arrow dalında tick başı movement'i
  yakalar; konumu önce bu movement ile ilerletir, rotation hedefini aynı
  movement'ten üretir ve ancak bundan sonra inertia/gravity uygular.
- `world/entity/projectile/arrow/AbstractArrow.java:243-252,323-349`,
  `Projectile.lerpRotation` ile current rotation'a `0.2F` wrapped yaklaşımını,
  `0.99F` air drag'ı ve `0.05` gravity'yi tanımlar. Pinned client jar
  bytecode'unda derece/radyan çarpanı `57.2957763671875` sabiti olarak
  gerçekleşir.
- `world/entity/projectile/Projectile.java:329-346`, inherited
  `lerpRotation` içindeki 180° sınır döngülerini ve `Mth.lerp` çağrısını;
  `world/entity/Entity.java:1510-1529`, `NoGravity` true olduğunda gravity'nin
  sıfırlanmasını tanımlar.

Snowball'dan farklı olarak Arrow dalında position ve rotation inertia/gravity
öncesindedir; rotation current `getXRot`/`getYRot` değerlerinden başlar. Oracle,
`Arrow` tabanlı özgün fixture ile bu branch'in gerekli Java `lerpRotation`,
`getAirDrag` ve `applyGravity` çağrılarını, collision ve level yan etkilerini
kontrollü olarak dışarıda bırakarak yürütür.

## Yeni dal ve varyant kararı

Bu dilim, P08.bl server-conformance kaydındaki Java `AbstractArrow.tick`
free-flight dalını ilk kez `client-parity` sınıfında çalıştırır. Snowball'ın
P08.aq.1 kaydından farklı gözlem; aynı başlangıç movement'iyle Arrow'un
position/rotation state'ini inertia ve gravity'den önce üretmesi, normal ve
`NoGravity` motion sırasının iki tick boyunca korunması ve `355° → 0°` yaw
geçişinde `Projectile.lerpRotation`'ın `-4°` wrapped state'ini vermesidir.
Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır.

Normal gravity ve `NoGravity` girdileri, wrapped yaw dalını da içeren tek
parametrik corpus kaydının iki entity satırı olarak tutuldu; sabit kopyası
olan yeni bir probe açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Arrow entity type ID
  `6`'dır. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)` içinde
  boş ve yüklü tek section'dır.
- Entity `37`, `(2,1,2)` konumunda `(0.25,0.5,0.25)` hareketi ve
  `(y=10,x=-20)` rotation'ı ile; entity `38`, `(4,1,2)` konumunda
  `(0,0,1)` hareketi ve `(y=355,x=30)` rotation'ı ile başlar. Entity `37`
  gravity kullanır, entity `38` `NoGravity` kullanır.
- Her world tick'inde previous position/rotation önce current state'ten
  alınır ve `tickCount` bir artırılır. Tick başındaki movement kopyası mevcut
  position'a eklenir; aynı movement'ten hedef yaw/pitch üretilip current
  rotation'a `Projectile.lerpRotation` ile `0.2F` yaklaşılır.
- Position ve rotation sonrasında üç motion bileşeni Java `0.99F` air drag
  ile çarpılır. `NoGravity=false` ise yalnız Y motion'ından `0.05` çıkarılır;
  `NoGravity=true` satırında gravity çıkarılmaz.
- Block/entity collision, in-ground metadata, water inertia, projectile hit,
  removal, particle/effect, gerçek packet/thread dispatch'i, renderer ve
  oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-arrow-tick` kaydı iki Arrow satırını ve iki tick işlemini
taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Java referans kimliği:
`666d09e1a916114383cc00942dcea61c5cbc8d0e6ca56e94cde7f26eed8290e3`

Math corpus'u **2992** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | Entity | position bileşeni | motion bileşeni | rotation Y | rotation X | tick |
| --- | ---: | --- | --- | --- | --- | ---: |
| initial | 37 | `y=3ff0000000000000` | `y=3fe0000000000000` | `41200000` | `c1a00000` | 0 |
| first-air-tick | 37 | `y=3ff8000000000000` | `y=3fdc7ae14ccccccd` | `41880000` | `c0a1b14c` | 1 |
| second-air-tick | 37 | `y=3fff1eb853333333` | `y=3fd8fec576fd2200` | `41b4cccc` | `40ca3fb4` | 2 |
| initial | 38 | `z=4000000000000000` | `z=3ff0000000000000` | `43b18000` | `41f00000` | 0 |
| first-air-tick | 38 | `z=4008000000000000` | `z=3fefae1480000000` | `c0800000` | `41c00000` | 1 |
| second-air-tick | 38 | `z=400feb8520000000` | `z=3fef5cfab6fd2200` | `c04ccccd` | `4199999a` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Yüklü air-cell
kontrolü type ID `6` için geçildiğinde `ClientEntity::tick_arrow`, position ve
rotation'ı current movement'ten alır; ardından Arrow inertia/gravity sırasını
uygular. Java bytecode'undaki derece/radyan sabiti ve `Projectile.lerpRotation`
wrapped döngüsü Rust'ta aynı işlem sırasıyla tutulur.

Debug adapter SHA-256:
`0f5f212da3a01f9497691100d9d4c59729929c0e6242559458779296017de0fe`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2992
difference=null
status=parity_passed
```

Release adapter SHA-256:
`4f6ab06ac16e77cc79c9101270241a1f6c05fe39629ba1c16efe07cac9ea6306`

Release adapter aynı capture üzerinde **2992/2992** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2992
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
Arrow AddEntity ve world tick sonucu [P08.bl server-conformance kaydında](client-world-entity-arrow-tick-conformance.md)
tutulur; canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Arrow block clip ve entity hit testi, in-ground/last-state lifecycle'ı, water
inertia, pickup/effect/particle state'i, diğer arrow türleri, general motion
integration, network entity dispatch, renderer ve oynanabilir istemci
bağlantısı açık kalır.
