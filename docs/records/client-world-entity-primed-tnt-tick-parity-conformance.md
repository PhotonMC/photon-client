# P08.bn.1 — ClientWorld Primed TNT free-flight tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki Primed TNT entity'sinin iki
ardışık `ClientWorld` tick'inde Java 26.2 ile Rust world kopyasında aynı konum,
hareket, rotation ve tick state'ini üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.bn server-conformance kaydı](client-world-entity-primed-tnt-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan Primed TNT AddEntity state'inin Rust
world tick'inde ilerletildiğini gösterir. O kayıt canlı sunucu wire ve kabul
kanıtıdır; bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/EntityTypes.java:981-990`, Primed TNT entity type ID'sini
  `133` olarak kaydeder. `client/multiplayer/ClientLevel.java:467-472`, uygun
  entity için eski transform'u kopyalar, `tickCount` değerini artırır ve
  entity tick gövdesini yürütür.
- `world/entity/item/PrimedTnt.java:99-107`, her Primed TNT tick'inde portal
  işlemi sonrasında gravity uygulandığını, hareketle konum alındığını ve
  hareketin air drag ile çarpıldığını belirtir. Bu sıra, Experience Orb'un
  client-only ilk tick korumasını içermez.
- `world/entity/item/PrimedTnt.java:105-107`, yere temas halinde ayrı bir
  hareket/bounce çarpanı uygular; bu kayıt geçiş ve hedef hücreleri boş hava
  olduğu için bu dalı çalıştırmaz.
- `world/entity/item/PrimedTnt.java:109-122`, fuse azaltma, removal, smoke ve
  explosion/lifecycle dallarını taşır. `PrimedTnt.java:94-96` gravity'yi
  `0.04`; `world/entity/Entity.java:1510-1529`, `NoGravity` true olduğunda
  gravity'nin sıfırlanmasını tanımlar.

Experience Orb ve Primed TNT aynı gravity/position/air-drag işlem sırasının
bir bölümünü paylaşsa da Java tick gövdeleri ayrıdır: Primed TNT ilk client
tick'inde entity gövdesini atlamaz ve her tick'te fuse/lifecycle mantığına
ilerler. Oracle, `PrimedTnt` tabanlı özgün fixture ile gerekli inherited
gravity ve air drag çağrılarını aynı sırada yürütür; portal, collision, block
effect, ground bounce, fuse ve explosion yan etkilerini kontrollü olarak
dışarıda bırakır.

## Yeni dal ve varyant kararı

Bu dilim, P08.bn server-conformance kaydındaki Primed TNT client tick dalını
ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; ilk world tick'inde
gravity → position → air drag sırasının doğrudan uygulanması, ikinci tick'te de
aynı hareket zincirinin sürmesi ve `NoGravity` satırında yalnız gravity'nin
atlanmasıdır. Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı
**0**'dır; Snowball, Arrow ve Experience Orb ayrı entity tick dallarını ölçer.

Normal gravity ve `NoGravity` girdileri tek parametrik corpus kaydının iki
entity satırı olarak verildi. Aynı Java dalının yalnız sabiti değişen kopyası
açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Primed TNT entity
  type ID `133`'tür. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)`
  içinde boş ve yüklü tek section'dır.
- Entity `37`, `(2,10,2)` konumunda `(0.125,0.25,-0.125)` hareketi ve
  `(y=10,x=20)` rotation'ı ile; entity `38`, `(4,10,2)` konumunda
  `(0,0,1)` hareketi ve `(y=45,x=30)` rotation'ı ile başlar. Entity `37`
  gravity kullanır, entity `38` `NoGravity` kullanır.
- Her world tick'inde ClientLevel eski position/rotation state'ini saklar,
  `tickCount` değerini artırır ve Primed TNT tick'i hareket zincirini yürütür.
  `NoGravity=false` satırında movement Y'den önce `0.04` çıkarılır.
- Entity gravity uygulanmış hareketle konum alır; sonra üç movement bileşeni
  Java f32 `0.98F` değerinin f64 işlemine genişletilmiş karşılığıyla çarpılır.
  `NoGravity=true` satırında gravity çıkarılmaz. Bu dar dalda rotation
  değişmez.
- Portal, block/entity collision, water/fluid effect, ground friction/bounce,
  fuse decrement, smoke particle, explosion, removal, block state, gerçek
  packet/thread dispatch'i, renderer ve oynanabilir istemci bu kaydın
  dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-primed-tnt-tick` kaydı iki Primed TNT satırını ve iki
tick işlemini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı
`null`'dır.

Java referans kimliği:
`2c3a05fb2a87c8b56f15ac055253aa479b5a8ef0d6ed42cfb01f849d27a3ceaa`

Math corpus'u **2994** kayıttır. Yeni kaydın seçili raw state bitleri:

| Adım | Entity | position Y/Z | motion Y/Z | rotation Y | rotation X | tick |
| --- | ---: | --- | --- | --- | --- | ---: |
| initial | 37 | `y=4024000000000000` | `y=3fd0000000000000` | `41200000` | `41a00000` | 0 |
| first-air-tick | 37 | `y=40246b851eb851ec` | `y=3fca57a78f5c28f5` | `41200000` | `41a00000` | 1 |
| second-air-tick | 37 | `y=4024c068dbae147b` | `y=3fc4cc46a4484947` | `41200000` | `41a00000` | 2 |
| initial | 38 | `z=4000000000000000` | `z=3ff0000000000000` | `42340000` | `41f00000` | 0 |
| first-air-tick | 38 | `z=4008000000000000` | `z=3fef5c2900000000` | `42340000` | `41f00000` | 1 |
| second-air-tick | 38 | `z=400fd70a40000000` | `z=3feebb98dbf48800` | `42340000` | `41f00000` | 2 |

## Rust doğrulaması

`ClientWorld::tick` production entity tick yolunu kullanır. Yüklü air-cell
kontrolü type ID `133` için geçildiğinde `ClientEntity::tick_primed_tnt`,
gravity → position → air friction sırasını her world tick'inde uygular.
Önceki position ve tick count state'leri aynı world tick preamble'ından
sorgulanır.

Debug adapter SHA-256:
`4da5f15e6e508a466a1d259404249893839957932dda424f9b48479a540b2276`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2994
difference=null
status=parity_passed
```

Release adapter SHA-256:
`13b6cc6c6e578d2e48403dfcf7c24b3fe0df7617d588fe6592223f866057518d`

Release adapter aynı capture üzerinde **2994/2994** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2994
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
Primed TNT AddEntity ve world tick sonucu [P08.bn server-conformance
kaydında](client-world-entity-primed-tnt-tick-conformance.md) tutulur; canlı
sunucu kabulü bu Java istemci state karşılaştırmasının yerine geçmez.

## Açık kapsam

Primed TNT fuse state'i ve fuse decrement, smoke particle, portal,
explosion/removal, block/entity collision, water/ground friction ve bounce,
block state, network entity dispatch, renderer, genel motion integration ve
oynanabilir istemci bağlantısı açık kalır.
