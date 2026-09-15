# P08.be.1 — ClientWorld `Entity.turn` passenger callback — [client-parity]

Bu kayıt, passenger üzerinde doğrudan `Entity.turn` çağrıldığında boat'ın
`onPassengerTurned` callback'inin Java istemcisindeki state'i Rust
`ClientWorld` kopyasında eşleştirmesini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.be server-conformance kaydı](client-world-entity-passenger-turn-conformance.md)
vanilla sunucusuna bağlı callback gözlemlerini tutar. O kayıt
`server-conformance` kanıtıdır; bu kayıt Java istemci turn state'ini
karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `world/entity/Entity.java:490-501`, `turn(double xo, double yo)` içinde
  mouse deltasını Java `float` olarak `xDelta = (float) yo * 0.15F` ve
  `yDelta = (float) xo * 0.15F` biçiminde üretir. Current x/y rotation'ı
  günceller, current ve previous pitch'i `[-90,90]` aralığında tutar ve bağlı
  vehicle varsa sonunda `vehicle.onPassengerTurned(this)` çağrısını yapar.
- `world/entity/vehicle/boat/AbstractBoat.java:677-688`, callback'in
  `clampRotation` yoluna girdiğini gösterir. Wrapped passenger/boat yaw farkı
  `[-105,105]` aralığına indirilir; correction current ve previous passenger
  yaw'a uygulanır. Passenger head yaw'ı current yaw'a, living passenger body
  yaw'ı boat yaw'ına yazılır.
- `world/entity/vehicle/boat/AbstractBoat.java:631-642`, tick içindeki
  `positionRider` dönüş bloğunda `CAN_TURN_IN_BOATS` istisnasının bulunduğunu
  gösterir. `onPassengerTurned` callback'i bu kontrolü tekrar kullanmaz;
  dolayısıyla Breeze doğrudan turn callback'inde clamp edilir.
- `world/entity/LivingEntity.java:238-241,498-501,3380-3392` Player ve
  Breeze'in head/body rotation setter state'ini taşır. `Entity.java:2969-2977`
  base Entity setter'larının no-op olduğunu; bu nedenle bu kayıt için iki
  living passenger satırının seçildiğini doğrular.

Oracle, gerçek vanilla `Boat`, remote `Player` ve `Breeze` sınıflarından Unsafe
ile allocate edilmiş fixture'lar kullanır. Entity kimliği, type, dimensions,
level, position, vehicle, movement ve transform history alanları kurulur;
fixture tick gövdeleri no-op'tur. Relation kurulduktan sonra gerçek Java
`Entity.turn` çağrısı çalıştırılır ve boat'ın gerçek
`onPassengerTurned` dispatch'i gözlenir. Breeze tag'i ayrıca bağlanmaz; bu
callback dalı `CAN_TURN_IN_BOATS` sorgusundan bağımsızdır.

## Yeni dal ve varyant kararı

Bu kayıt, P08.be server-conformance kaydındaki `Entity.turn` → boat
`onPassengerTurned` dalını ilk kez Java istemci state'iyle `client-parity`
olarak çalıştırır. Yeni gözlem, tick sırasında Breeze için atlanan
`CAN_TURN_IN_BOATS` koşulunun doğrudan turn callback'inde atlanmaması ve
callback'in passenger konumunu değiştirmeden current/previous rotation ile
living head/body state'ini güncellemesidir. Mevcut aynı sonucu veren
`client-parity` kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı iki branch witness'ı taşır: Player type ID `156`
ve Breeze type ID `17`. Aynı callback kod yolunun sabit değiştiren kopyaları
açılmadı; Player living dispatch'i, Breeze ise P08.bb tick exemption'ından
farklı callback sonucunu aynı turn operation içinde görünür kılar.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde iki oak boat type ID `89`, konum `(2,2,2)`, yaw `0.0F` ve
  `NoGravity=true` kullanılır.
- Her boat tek passenger taşır. Player type ID `156` ve Breeze type ID `17`,
  konum `(2,2,2)`, zero movement, yaw `168.75F`, pitch `0.0F` ve
  `NoGravity=true` ile başlar.
- `turn` operation her passenger için `mouse_x=-100.0`, `mouse_y=0.0`
  uygular. Java f32 mouse delta'sı current ve previous yaw'a eklenir; callback
  clamp'i sonucu current ve previous passenger yaw `105.0F` olur.
- Callback sonrası Player ve Breeze current head yaw `105.0F`, previous head
  yaw `168.75F`, current body yaw `0.0F` ve previous body yaw `168.75F` olarak
  gözlenir. Passenger position, previous position, movement ve tick count
  değişmez.
- State'ler boat ve passenger entity tick count, position, previous position,
  movement, current/previous yaw-pitch ve no-gravity alanlarını taşır. Living
  passenger dispatch'i nedeniyle head/body yaw alanları da karşılaştırılır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-turn` kaydı Player ve Breeze satırlarını aynı
initial ve `direct-passenger-turn` state'leri içinde yürütür. Java oracle iki
çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Seçili Java state'leri:

| Passenger | State | Position | Current / previous yaw | Head current / previous | Body current / previous |
| --- | --- | --- | --- | --- | --- |
| Player `156` | initial | `(2,2,2)` | `168.75 / 168.75` | `168.75 / 168.75` | `168.75 / 168.75` |
| Player `156` | direct-passenger-turn | `(2,2,2)` | `105.0 / 105.0` | `105.0 / 168.75` | `0.0 / 168.75` |
| Breeze `17` | initial | `(2,2,2)` | `168.75 / 168.75` | `168.75 / 168.75` | `168.75 / 168.75` |
| Breeze `17` | direct-passenger-turn | `(2,2,2)` | `105.0 / 105.0` | `105.0 / 168.75` | `0.0 / 168.75` |

Math capture kimliği:
`3fea069779006719563e15c97ef82ea5d34f74763a220eb632a8b9ec58c188d5`

Bu capture math ailesinde **3014** case içerir. Tam corpus Java capture
kimliği:
`9327093e42bcd823623a6a098c82488f7029187615ee42c2ee1db517d0149b80`

Tam capture **3875** case içerir; iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::turn_entity` yollarını
kullanır. Player ve Breeze state'leri aynı ordered entity listesinde tutulur;
turn operation her passenger için production `ClientWorld::turn_entity`
üzerinden uygulanır. Clamp, body/head dispatch'i, previous rotation history'si
ve position'ın korunması Rust world kodundan gelir.

Beklenen rotation ve history değerleri Java capture'dan alındı; Rust
çıktısından türetilmedi. `src/world.rs` içinde bu dilim için yeni bir davranış
dalı açılmadı; mevcut `turn_entity` ve boat callback kopyası Java oracle ile
karşılaştırıldı.

Debug adapter SHA-256:
`5b4f6752f4681cf90b09b4adea9aab21f93ff30821c6923b280c2bb10a3155db`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3014
difference=null
status=parity_passed
```

Tam debug karşılaştırması da `case_count=3875`, `difference=null` ve
`status=parity_passed` verdi.

Release adapter SHA-256:
`815a9ac90304da8a1fdbeafd2becc2a56c1ee7497efe0b5b5e094578bd9abe5b`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3875
difference=null
status=parity_passed
```

`cargo test --locked --offline` sonucu **194 passed**,
`cargo clippy --all-targets --all-features --locked --offline -- -D warnings`
sonucu temizdir. `cargo fmt --all -- --check`, Python syntax check ve
`git diff --check` de geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun `Entity.turn` sonrası
callback state'i [P08.be server-conformance kaydında](client-world-entity-passenger-turn-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Nonzero boat `deltaRotation`, gerçek mouse input callback'inden `turn_entity`
yoluna bağlanması, turn operation'ın nested passenger'a yayılması, boat dışı
vehicle callback'i, dismount, collision, render/frame timing'i ve gerçek
oynanabilir istemci bağlantısı açık kalır. Boat tick clamp'i [P08.ba.1
client-parity kaydında](client-world-entity-passenger-rotation-parity-conformance.md),
Breeze tick istisnası [P08.bb.1 kaydında](client-world-entity-passenger-boat-turn-exemption-parity-conformance.md)
ve Animal rotation/position dalları [P08.bc.1](client-world-entity-passenger-animal-rotation-parity-conformance.md)
ile [P08.bd.1](client-world-entity-passenger-animal-position-parity-conformance.md)
kayıtlarında izlenir. Bu kayıt P08'in `client-parity` kapanışı değildir.
