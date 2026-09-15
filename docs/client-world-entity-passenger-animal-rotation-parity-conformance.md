# P08.bc.1 — ClientWorld Animal passenger rotation offset — [client-parity]

Bu kayıt, tam kapasitedeki oak boat içindeki `Animal` passenger'ının Java
istemcisindeki living body/head rotation ve entity ID parite offset'inin Rust
`ClientWorld` kopyasında eşleşmesini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.bc server-conformance kaydı](client-world-entity-passenger-animal-rotation-conformance.md)
vanilla sunucusuna bağlı Animal rotation gözlemlerini tutar. O kayıt
`server-conformance` kanıtıdır; bu kayıt Java istemci tick state'ini
karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `client/multiplayer/ClientLevel.java:461-491`, boat root entity'sini tick
  eder ve ilişkisi korunan passenger'ları ordered passenger dalında yürütür.
- `world/entity/Entity.java:2380-2417`, passenger `rideTick` sonrasında
  vehicle'ın `positionRider` çağrısını çalıştırır. Passenger hareketi
  sıfırlanır; seat konumundan passenger `VEHICLE` attachment noktası çıkarılır.
- `world/entity/vehicle/boat/AbstractBoat.java:631-642`,
  `CAN_TURN_IN_BOATS` tag'inde olmayan passenger'a `deltaRotation` ekler ve
  `clampRotation` çağrısını yapar. `AbstractBoat.java:677-684` wrapped yaw
  farkını `[-105,105]` aralığına indirir.
- Aynı `positionRider` bloğunda living passenger için body yaw boat yaw'ına,
  head yaw clamp edilmiş current yaw'a yazılır. Passenger `Animal` ve ordered
  liste normal boat kapasitesine ulaştığında entity ID çiftse `90`, tekse `270`
  derece hem body hem head yaw'a eklenir.
- `world/entity/Entity.java:2969-2977` base entity'deki head/body setter'larının
  no-op olduğunu; `world/entity/LivingEntity.java:238-241,498-501,3380-3392`
  living entity'nin current/old head ve body state'ini tuttuğunu doğrular.
  `AbstractBoat.java:755-757` normal boat kapasitesini `2` yapar.
- `world/entity/EntityAttachments.java:44-80` ve
  `world/entity/animal/Animal.java:42-49`, bu Cow fixture'ında default feet
  attachment'ının kullanıldığını gösterir. `EntityTypeIds.java:12-164`, Cow
  protocol type ID'sini `30` olarak verir.

Oracle gerçek vanilla `Cow` sınıfından Unsafe ile allocate edilmiş bir alt sınıf
kullanır. Entity ID, type, dimensions, level, position, vehicle ve transform
history alanları kurulur; fixture `tick()` gövdesi no-op'tur. Kalıtılan Java
`LivingEntity`/`Entity` `rideTick` ve gerçek `Boat.positionRider` dispatch'i
çalışır. İkinci Snowball passenger'ı normal oak boat kapasitesini doldurur;
Snowball'ın tick state'i de aynı capture'da gözlenir.

## Yeni dal ve varyant kararı

Bu kayıt, P08.bc server-conformance kaydındaki `AbstractBoat.positionRider`
Animal alt dalını ilk kez Java istemci tick akışıyla `client-parity` olarak
çalıştırır. Yeni gözlem, living clamp sonrasında tam kapasitede entity ID
paritesine göre body/head rotation'a `90` veya `270` eklenmesidir. Mevcut aynı
sonucu veren `client-parity` kayıtlarının sayısı **0**'dır.

Sabit değiştiren Animal type ID kopyaları açılmadı. Tek parametrik corpus
kaydındaki iki tablo satırı Cow type ID `30` için çift ve tek entity ID dallarını
kapatır; filler passenger yalnız normal boat kapasite koşulunu sağlar.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, konum `(2,2,2)`, yaw `0.0F` ve
  `NoGravity=true` kullanılır.
- Her satırda Cow type ID `30`, başlangıçta `(2,12,2)` konumunda, zero
  movement, yaw `170.0F`, pitch `0.0F` ve `NoGravity=true` ile ilk passenger'dır.
  Snowball type ID `120` ikinci passenger'dır; böylece ordered list normal
  oak boat kapasitesi `2` olur.
- Bir world tick'inde boat root, Cow passenger ve filler sırasıyla işlenir.
  Cow'ın seat position'ı `(2,2.1875,2.4000000059604645)` olur; previous
  position `(2,12,2)` olarak kalır. Filler'ın ikinci seat position'ı
  `(2,2.1875,1.399999976158142)` olur.
- Boat yaw `0.0F` ve delta rotation sıfır olduğu için Cow current/previous yaw
  `105.0/105.0` olur. Çift ID'li Cow'da current head/body `195.0/90.0`, tek
  ID'li Cow'da `375.0/270.0` olur. Her iki satırda previous head/body
  `170.0/170.0`'dir.
- State'ler entity tick count, position, previous position, movement, current /
  previous yaw-pitch ve no-gravity alanlarını taşır. Cow living dispatch'i
  nedeniyle head ve body yaw alanları da taşınır; filler base entity alanlarıyla
  kalır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-animal-rotation` kaydı iki parametrik satırı
aynı initial ve `animal-capacity-offset` state'leri içinde yürütür. Java oracle
iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Seçili Java state'i:

| Satır | State | Cow ID | Cow position | Current / previous yaw | Head current / previous | Body current / previous |
| --- | --- | ---: | --- | --- | --- | --- |
| even-animal | initial | `52` | `(2,12,2)` | `170.0 / 170.0` | `170.0 / 170.0` | `170.0 / 170.0` |
| even-animal | animal-capacity-offset | `52` | `(2,2.1875,2.4000000059604645)` | `105.0 / 105.0` | `195.0 / 170.0` | `90.0 / 170.0` |
| odd-animal | initial | `55` | `(2,12,2)` | `170.0 / 170.0` | `170.0 / 170.0` | `170.0 / 170.0` |
| odd-animal | animal-capacity-offset | `55` | `(2,2.1875,2.4000000059604645)` | `105.0 / 105.0` | `375.0 / 170.0` | `270.0 / 170.0` |

Math capture kimliği:
`d0c55afd18ea1edf5d6add5583a2e62bfd9f676e31610e1b238f9ec16c8365ca`

Bu capture math ailesinde **3012** case içerir. Tam corpus Java capture
kimliği:
`c64de491ee85c047a6676c2b08b36221ae26b370a36f6908b50e6651765ca2bb`

Tam capture **3873** case içerir; iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::tick` yollarını
kullanır. Cow type ID `30` için living head/body state'i adapter çıktısına
eklenir; seat position, movement reset'i, tick history'si ve Animal parite
offset'i production world kodundan gelir.

Beklenen position, rotation ve history değerleri Java capture'dan alındı; Rust
çıktısından türetilmedi. `src/world.rs` içinde bu dilim için yeni bir davranış
dalı açılmadı; mevcut Animal type tablosu ve kapasite clamp yolu karşılaştırıldı.

Debug adapter SHA-256:
`793eb634a24e14eab83deaa11c434241fc4a5f6e52a7576cf291a5d8c2c6c128`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3012
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3873
difference=null
status=parity_passed
```

Release adapter SHA-256:
`4f6466f0e3a2c5b12885f12a94165f1717bba4a94e9b86fc09e9ac997b70b38f`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3873
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
packet'lerini kabul etmesi [P08.bc server-conformance kaydında](client-world-entity-passenger-animal-rotation-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Nonzero boat `deltaRotation`, tam kapasitede olmayan Animal, chest boat kapasite
dalı, Cow dışındaki pinned Animal type ID'leri, özel vehicle attachment,
`Entity.turn` callback'i, nested controlling passenger, collision, dismount,
gerçek oynanabilir istemci bağlantısı ve render/frame timing'i açık kalır.
Animal attachment ve iki passenger seat offset'i [P08.bd kaydında](client-world-entity-passenger-animal-position-conformance.md),
genel boat clamp [P08.ba.1 kaydında](client-world-entity-passenger-rotation-parity-conformance.md)
ve Breeze tag istisnası [P08.bb.1 kaydında](client-world-entity-passenger-boat-turn-exemption-parity-conformance.md)
izlenir. Bu kayıt P08'in `client-parity` kapanışı değildir.
