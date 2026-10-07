# P08.bd.1 — ClientWorld Animal passenger vehicle attachment — [client-parity]

Bu kayıt, tek passenger olan Cow'ın Java istemcisindeki default vehicle
attachment ve boat seat konumunun Rust `ClientWorld` kopyasında eşleşmesini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java
oracle'ına ve production Rust adapter'a verilir.

[P08.bd server-conformance kaydı](client-world-entity-passenger-animal-position-conformance.md)
vanilla sunucusuna bağlı Animal attachment ve seat gözlemlerini tutar. O kayıt
`server-conformance` kanıtıdır; bu kayıt Java istemci tick state'ini
karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `client/multiplayer/ClientLevel.java:461-491`, boat root entity'sini tick
  eder ve passenger'ı ordered passenger dalında yürütür.
- `world/entity/Entity.java:2380-2417`, passenger `rideTick` sonrasında
  vehicle'ın `positionRider` çağrısını çalıştırır. Passenger movement'i
  sıfırlanır ve tick başındaki position history korunur.
- `world/entity/Entity.java:2403-2405`, vehicle riding position'ından
  passenger'ın `VEHICLE` attachment'ını çıkarır; attachment yoksa feet
  fallback'i `(0,0,0)` kullanılır.
- `world/entity/vehicle/boat/AbstractBoat.java:132-150`, tek passenger
  kolunda `getSinglePassengerXOffset()` değerini kullanır. Cow için bu değer
  sıfırdır; iki passenger'lı yoldaki ilk seat `0.2F` ve Animal `+0.2F`
  offset'leri bu tek passenger kolunda çalışmaz.
- `world/entity/vehicle/boat/Boat.java:15-17`, oak boat ride height'ını
  dimensions yüksekliğinin üçte biri olarak hesaplar; sonuç `0.1875`'tir.
  `world/entity/EntityTypeIds.java:12-164`, Cow protocol type ID'sini `30`
  olarak verir.

Oracle, önceki P08.bc.1 ile aynı gerçek vanilla `Cow` alt sınıfı fixture'ını
Unsafe ile allocate eder. Entity type/dimensions, position, vehicle ve
transform history alanları kurulur; fixture `tick()` gövdesi no-op bırakılır.
Kalıtılan Java `LivingEntity`/`Entity` `rideTick` ile gerçek
`Boat.positionRider` dispatch'i çalışır. Bu kayıtta boat passenger listesinde
yalnız Cow vardır.

## Yeni dal ve varyant kararı

Bu kayıt, P08.bd server-conformance kaydındaki `AbstractBoat.positionRider`
tek passenger kolunu ilk kez Java istemci tick akışıyla `client-parity` olarak
çalıştırır. Yeni gözlem, default feet attachment ile Cow konumunun
`(2,2.1875,2)` olmasıdır; P08.bc.1'deki tam kapasite ilk seat sonucu
`z=2.4000000059604645` değerinden ayrılır. Bu tek passenger sonucu veren
mevcut `client-parity` kayıtlarının sayısı **0**'dır.

Sabit değiştiren ayrı Animal type ID satırları açılmadı. Tek parametrik corpus
kaydı Cow type ID `30` için attachment fallback'ini ve tek passenger seat
dalını kapatır; tam kapasite Animal offset'i P08.bc.1'de karşılaştırılır.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, konum `(2,2,2)`, yaw `0.0F` ve
  `NoGravity=true` kullanılır.
- Cow type ID `30`, başlangıçta `(2,12,2)` konumunda, zero movement, yaw
  `170.0F`, pitch `0.0F` ve `NoGravity=true` ile tek ordered passenger'dır.
- Bir world tick'inde boat root ve Cow passenger işlenir. Cow movement'i
  `(0,0,0)` olur; current position `(2,2.1875,2)` ve previous position
  `(2,12,2)` olarak gözlenir.
- Boat yaw `0.0F` olduğundan seat offset vehicle yaw'ıyla dönmez. Cow'ın
  default feet attachment'ı seat y'sinden ek bir değer düşürmez; tek
  passenger kolunda Animal `+0.2F` çoklu-seat z offset'i uygulanmaz.
- State entity tick count, position, previous position, movement, current /
  previous yaw-pitch ve no-gravity alanlarını taşır. Living head/body alanları
  bu konum kaydının çıktısına eklenmez; living rotation state'i P08.bc.1'de
  karşılaştırılır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-animal-position` kaydı bir initial ve bir
`single-animal-seat` state'i yürütür. Java oracle iki çalıştırmada kararlı
kaldı; tekrar farkı `null`'dır.

Seçili Java state'i:

| State | Cow ID | Cow position | Previous position | Current / previous yaw | Motion |
| --- | ---: | --- | --- | --- | --- |
| initial | `58` | `(2,12,2)` | `(2,12,2)` | `170.0 / 170.0` | `(0,0,0)` |
| single-animal-seat | `58` | `(2,2.1875,2)` | `(2,12,2)` | `105.0 / 105.0` | `(0,0,0)` |

Math capture kimliği:
`0c066bd023cd20df347727af45bdcc2890396174ec3f56083ab6c239b28d67f2`

Bu capture math ailesinde **3013** case içerir. Tam corpus Java capture
kimliği:
`183841caf8177ed9c190fe7e7f82ac2bfcfe9ffb1d5eae1bcf64f2edc82d9099`

Tam capture **3874** case içerir; iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::tick` yollarını
kullanır. Cow type ID `30` için default attachment fallback'i ve tek
passenger seat hesabı production `position_entity_passenger` yolundan gelir;
adapter beklenen konumu Java capture'dan türetilmiş bir sabit olarak
uygulamaz.

Beklenen position, rotation ve history değerleri Java capture'dan alındı; Rust
çıktısından türetilmedi. `src/world.rs` içinde bu dilim için yeni bir davranış
dalı açılmadı.

Debug adapter SHA-256:
`fe39d9121093d1520b86b114a31fc6abd8eed7bc49b987fb4f9e95526e8fe06f`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3013
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3874
difference=null
status=parity_passed
```

Release adapter SHA-256:
`fff2f4604b2a878d340e077c90026b9c5ebf8eb9a951104fcd518936862803a0`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3874
difference=null
status=parity_passed
```

`cargo test --locked --offline` sonucu **194 passed**,
`cargo clippy --all-targets --all-features --locked --offline -- -D warnings`
sonucu temizdir. `cargo fmt --all -- --check`, Python syntax check ve
`git diff --check` de geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun Animal passenger
packet'lerini kabul etmesi [P08.bd server-conformance kaydında](client-world-entity-passenger-animal-position-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Tam kapasite Animal z offset'i [P08.bc.1 kaydında](client-world-entity-passenger-animal-rotation-parity-conformance.md),
chest boat kapasite ve family geometry [P08.az.1 kaydında](client-world-entity-passenger-boat-family-position-parity-conformance.md),
Player özel attachment [P08.ay.1 kaydında](client-world-entity-passenger-player-position-parity-conformance.md)
izlenir. Pose/scale/dimensions'a bağlı attachment, özel Animal passenger
attachment'ı, nonzero boat `deltaRotation`, rotation callback'i, nested
passenger, collision, dismount, gerçek oynanabilir istemci bağlantısı ve
render/frame timing'i açık kalır. Bu kayıt P08'in `client-parity` kapanışı
değildir.
