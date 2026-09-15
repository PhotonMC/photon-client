# P08.bo.1 — ClientWorld FallingBlock free-flight tick — [client-parity]

Bu kayıt, yüklü ve boş bir world section içindeki FallingBlock entity'sinin iki
ardışık `ClientWorld` tick'inde Java 26.2 ile Rust world kopyasında aynı konum,
hareket, rotation ve tick state'ini üretmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.bo server-conformance kaydı](client-world-entity-falling-block-tick-conformance.md)
vanilla 26.2 dedicated server'dan alınan FallingBlock AddEntity state'inin Rust
world tick'inde kabul edilmesini gösterir. O kayıt canlı sunucu wire kanıtıdır;
bu kayıt Java istemci davranışını aynı girdide karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/EntityTypes.java:448-455`, FallingBlock entity type ID'sini
  `51` olarak kaydeder. `client/multiplayer/ClientLevel.java:467-472`, uygun
  entity için eski transform'u kopyalar, `tickCount` değerini artırır ve
  entity tick gövdesini yürütür.
- `world/entity/item/FallingBlockEntity.java:148-157`, block state hava değilse
  `time` değerini artırır, gravity uygular ve hareketle konum alır. Client
  tarafında server-only placement/drop gövdesi yürütülmese de bu hareket dalı
  çalışır.
- `world/entity/item/FallingBlockEntity.java:247-249`, hareket adımından sonra
  inherited `getAirDrag()` ile `0.98F` air drag uygular. Bu kaynak dalı, önceki
  server-conformance probe'unda hareketin korunması şeklinde kaydedilen
  gözlemden farklıdır; server-conformance wire kabulü Java client state'ini
  doğrulamaz.
- `world/entity/item/FallingBlockEntity.java:158-245`, collision, ground,
  block placement, drop ve server lifecycle dallarını; `Entity.java:1510-1529`,
  `NoGravity` true olduğunda gravity'nin sıfırlanmasını tanımlar.

FallingBlock'un ayırt edici dalı, gravity/position sonrasında air drag'ın
`FallingBlockEntity.tick` gövdesinin sonunda uygulanmasıdır. Oracle,
`FallingBlockEntity` tabanlı özgün fixture ile `applyGravity`, position adımı
ve inherited `getAirDrag` çağrılarını aynı sırada yürütür; block-state çözümü,
collision, portal, ground bounce ve lifecycle yan etkilerini kontrollü olarak
dışarıda bırakır.

## Yeni dal ve varyant kararı

Bu dilim, P08.bo server-conformance kaydındaki FallingBlock client tick dalını
ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem; her tick'te
gravity → position → `0.98F` air drag sırasının yürütülmesi ve `NoGravity`
satırında yalnız gravity'nin atlanmasıdır. Aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; Snowball, Arrow, Experience Orb
ve Primed TNT ayrı entity tick gövdelerini ölçer.

Normal gravity ve `NoGravity` girdileri tek parametrik corpus kaydının iki
entity satırı olarak verildi. Aynı Java dalının yalnız sabiti değişen kopyası
açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve FallingBlock entity
  type ID `51`'dir. Başlangıç world'ü dimension min Y `0` olan, chunk `(0,0)`
  içinde boş ve yüklü tek section'dır.
- Entity `37`, `(2,10,2)` konumunda `(0.125,0.25,-0.125)` hareketi ve
  `(y=10,x=20)` rotation'ı ile; entity `38`, `(4,10,2)` konumunda
  `(0,0,1)` hareketi ve `(y=45,x=30)` rotation'ı ile başlar. Entity `37`
  gravity kullanır, entity `38` `NoGravity` kullanır.
- Her world tick'inde ClientLevel eski position/rotation state'ini saklar,
  `tickCount` değerini artırır ve FallingBlock tick'i `time` değerini artırır.
  `NoGravity=false` satırında movement Y'den önce `0.04` çıkarılır.
- Entity gravity uygulanmış hareketle konum alır; sonra üç movement bileşeni
  Java f32 `0.98F` değerinin f64 işlemine genişletilmiş karşılığıyla çarpılır.
  `NoGravity=true` satırında gravity çıkarılmaz. Bu dar dalda rotation
  değişmez.
- Block state/metadata ve `time`'ın server lifecycle kullanımı, block/entity
  collision, water/fluid effect, portal, ground friction/bounce, landing,
  block placement, drop/removal, block entity data, gerçek packet/thread
  dispatch'i, renderer ve oynanabilir istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-falling-block-tick` kaydı iki FallingBlock satırını ve iki
tick işlemini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı
`null`'dır.

Java referans kimliği:
`6407e33e94bd1616321dd28beab63ca3ca183fa97e4f29c664ed5bd33474595f`

Math corpus'u **2995** kayıttır. Yeni kaydın seçili raw state bitleri:

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
kontrolü type ID `51` için geçildiğinde `ClientEntity::tick_falling_block`,
gravity → position → air friction sırasını her world tick'inde uygular.
FallingBlock için Java kaynakta bulunan inherited `0.98F` air drag, Rust'ta
f32 sabitinin f64 genişletmesi olarak uygulanır. Önceki position ve tick count
state'leri aynı world tick preamble'ından sorgulanır.

Debug adapter SHA-256:
`f95067fa6a8810a6a87ae9cf96d84f1761a2149b772102b185a110a323b0f4bf`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2995
difference=null
status=parity_passed
```

Release adapter SHA-256:
`78e7ec1b464951493a42d4a714d43d0c3c3c588b1f81de0484ab3b519a1cfb8f`

Release adapter aynı capture üzerinde **2995/2995** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=2995
difference=null
status=parity_passed
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki
FallingBlock AddEntity ve wire kabul sonucu [P08.bo server-conformance
kaydında](client-world-entity-falling-block-tick-conformance.md) tutulur;
canlı sunucu kabulü bu Java istemci state karşılaştırmasının yerine geçmez.

## Açık kapsam

FallingBlock block-state/metadata çözümü, `time` ile server lifecycle, collision
ve fluid etkileri, portal, landing/placement, bounce/drop/removal lifecycle'ı,
block entity verisi, network entity dispatch, renderer, genel motion
integration ve oynanabilir istemci bağlantısı açık kalır.
