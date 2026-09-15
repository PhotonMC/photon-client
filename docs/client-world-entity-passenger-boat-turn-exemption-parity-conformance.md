# P08.bb.1 — ClientWorld Breeze boat turn exemption — [client-parity]

Bu kayıt, Breeze passenger'ının Java istemcisindeki boat
`CAN_TURN_IN_BOATS` istisnasının Rust `ClientWorld` kopyasında eşleşmesini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java
oracle'ına ve production Rust adapter'a verilir.

[P08.bb server-conformance kaydı](client-world-entity-passenger-boat-turn-exemption-conformance.md)
vanilla sunucusuna bağlı Breeze rotation gözlemini tutar. O kayıt
server-conformance kanıtıdır; bu kayıt Java istemci tick state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `client/multiplayer/ClientLevel.java:461-491` kök entity'yi tick eder ve
  ilişkisi korunan passenger'ı ayrı passenger dalında yürütür.
- `world/entity/Entity.java:2380-2417`, passenger `rideTick` sonrasında bağlı
  vehicle'ın `positionRider` yolunu çağırır. Passenger hareketi sıfırlanır;
  vehicle riding position'ı passenger `VEHICLE` attachment'ından çıkarılır.
- `world/entity/vehicle/boat/AbstractBoat.java:631-642`, passenger
  `EntityTypeTags.CAN_TURN_IN_BOATS` içindeyse `deltaRotation` ekleyen,
  head yaw'ı güncelleyen ve `clampRotation` çağıran bloğu atlar. Genel seat ve
  attachment position hesabı bu koşuldan bağımsız devam eder.
- `tags/EntityTypeTags.java:25-27` tag anahtarını tanımlar;
  `data/tags/EntityTypeTagsProvider.java:178-184` pinned vanilla verisinde
  bu tag'e yalnız Breeze'i ekler. `world/entity/EntityTypes.java:246-249`
  Breeze boyutlarını `0.6F x 1.77F` olarak kaydeder; `EntityTypeIds.java:25`
  resource key'i ve protocol registry type ID `17`'yi verir.
- `world/entity/Entity.java:368-369` entity tag sorgusunu type holder'a
  yönlendirir. Oracle, `Bootstrap` sonrasında
  `EntityType` tag reload yoluyla Breeze holder'ını bu tag'e bağlar; böylece
  ölçülen dal gerçek `passenger.is(CAN_TURN_IN_BOATS)` sonucudur.
- Oracle fixture'ı gerçek vanilla `Breeze` alt sınıfını Unsafe ile yalnızca
  gerekli entity/living state alanlarını kurarak oluşturur. `tick` no-op'tur;
  kalıtılan Java `LivingEntity`/`Entity` `rideTick` sırası ve gerçek
  `Boat.positionRider` dispatch'i çalışır. `LivingEntity` setter'ları
  head/body rotation state'ini gözlemlenebilir kılar.

## Yeni dal ve varyant kararı

Bu kayıt P08.ba.1'den sonra `AbstractBoat.positionRider` içindeki
`CAN_TURN_IN_BOATS` tag skip dalını ilk kez Java client tick akışıyla
`client-parity` olarak çalıştırır. Farklı gözlem, Breeze'in non-tag passenger
gibi `[-105,105]` clamp'ine girmemesi ve current/previous yaw ile living
head/body rotation'larını korumasıdır. Mevcut aynı sonucu veren
`client-parity` kayıtlarının sayısı **0**'dır.

Tek parametrik corpus satırı pinned tag üyesi Breeze type ID `17`'yi temsil
eder. Tag tablosundaki aynı davranışın sabit değiştiren kopyaları veya başka
bir type ID satırı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, konum `(2,2,2)`, yaw `0.0F` ve
  `NoGravity=true` kullanılır.
- Breeze type ID `17`, başlangıçta `(2,12,2)` konumunda, zero movement, yaw
  `170.0F`, pitch `0.0F` ve `NoGravity=true` ile boat'a bağlıdır. Bir world
  tick'i vehicle root ve passenger sırasını çalıştırır.
- Java seat position'ı oak boat için `(2,2.1875,2)` olur. Breeze'in default
  vehicle attachment'ı sıfır olduğu için passenger position bundan ayrıca
  düşürülmez; previous position `(2,12,2)` olarak kalır.
- Tag skip sonrasında Breeze current ve previous yaw `170.0F`, current ve
  previous head yaw `170.0F`, current ve previous body yaw `170.0F` olarak
  kalır. Boat'ın `deltaRotation` değeri sıfır olduğundan bu capture'da ek
  rotation etkisi yoktur; istisnanın ölçülen sonucu clamp'in uygulanmamasıdır.
- Her state entity tick count, position, previous position, movement, current /
  previous yaw-pitch ve no-gravity alanlarını taşır. Breeze living dispatch'i
  nedeniyle head ve body yaw alanları da taşınır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-boat-turn-exemption` kaydı oak boat ve Breeze
fixture'ını bir initial ve bir tick state'inde yürütür. Java oracle iki
çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Seçili Java state'i:

| State | Passenger position | Previous position | Current / previous yaw | Head current / previous | Body current / previous |
| --- | --- | --- | --- | --- | --- |
| initial | `(2,12,2)` | `(2,12,2)` | `170.0 / 170.0` | `170.0 / 170.0` | `170.0 / 170.0` |
| breeze-exemption | `(2,2.1875,2)` | `(2,12,2)` | `170.0 / 170.0` | `170.0 / 170.0` | `170.0 / 170.0` |

Math capture kimliği:
`4a0cbc1f37519d78411dc112cbc306d026f5745052aaa2373b0d9f11ab4e92f1`

Bu capture math ailesinde **3011** case içerir. Tam corpus Java capture
kimliği:
`1508ffde0d078b8250d7f8c73d21afeed6c7df1858233c9b41d8f669a7998d5b`

Tam capture **3872** case içerir; iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::tick` yollarını
kullanır. Type ID `17` için production
`position_entity_passenger` seat/attachment hesabını uygular ve
`clamp_entity_passenger_rotation` Java tag skip dalını döndürür. Head/body
alanları yalnız Java capture'da yaşayan Breeze state'ini görünür kılmak için
adapter çıktısına eklenir; rotation sonucu Rust world state'inden gelir.

Beklenen position, rotation ve history değerleri Java capture'dan alındı;
Rust çıktısından türetilmedi.

Debug adapter SHA-256:
`80d41cdc7a5d09428df5c85c4836881075f9ad942a9497f550f0b9dc01a0d962`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3011
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3872
difference=null
status=parity_passed
```

Release adapter SHA-256:
`2f5de399c29af3fdaafb2c3efdabb6c514a67a87b66f82eedbc9654307359f18`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3872
difference=null
status=parity_passed
```

`cargo test --locked --offline` sonucu **194 passed**,
`cargo clippy --all-targets --all-features --locked --offline -- -D warnings`
sonucu temizdir. `cargo fmt --all -- --check`, Python syntax check ve
`git diff --check` de geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun Breeze passenger
packet'lerini kabul etmesi [P08.bb server-conformance kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Nonzero boat `deltaRotation`, `Entity.turn` callback'i,
`CAN_TURN_IN_BOATS` tag'inin dinamik network güncellemesi, Animal kapasite
rotation offset'i, nested controlling passenger, chest boat/raft rotation,
collision, dismount, Breeze AI, gerçek oynanabilir istemci bağlantısı ve
render/frame timing'i açık kalır. Breeze default vehicle attachment
[P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md),
genel boat rotation clamp [P08.ba.1 kaydında](client-world-entity-passenger-rotation-parity-conformance.md)
izlenir. Bu kayıt P08'in `client-parity` kapanışı değildir.
