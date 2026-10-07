# P08.ba.1 — ClientWorld boat passenger rotation clamp — [client-parity]

Bu kayıt, oak boat passenger'ının Java istemcisindeki rotation clamp yolunun
Rust `ClientWorld` kopyasında eşleşmesini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.ba server-conformance kaydı](client-world-entity-passenger-rotation-conformance.md)
vanilla sunucusuna bağlı boat passenger rotation gözlemlerini tutar. O kayıt
server-conformance kanıtıdır; bu kayıt Java istemci tick state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `client/multiplayer/ClientLevel.java:461-491` kök entity'yi tick eder ve
  ilişkisi korunan passenger'ı ayrı passenger dalında yürütür.
- `world/entity/Entity.java:2380-2417`, passenger `rideTick` sonrasında bağlı
  vehicle'ın `positionRider` yolunu çağırır. Passenger'ın hareketi bu akışta
  sıfırlanır ve vehicle attachment noktası seat konumundan çıkarılır.
- `world/entity/vehicle/boat/AbstractBoat.java:631-642`, `CAN_TURN_IN_BOATS`
  tag'inde olmayan passenger için `deltaRotation` eklenmesinden sonra clamp
  yolunu çalıştırır. `AbstractBoat.java:677-684` wrapped yaw farkını
  `[-105,105]` aralığına indirir ve correction'ı current ile old yaw'a uygular.
- Aynı `clampRotation` çağrısı head yaw'ı current yaw'a, living passenger'ın
  body yaw'ını boat yaw'ına yazar. Genel `Entity` setter'ları no-op olduğu için
  base Snowball satırı body/head alanı üretmez; `Player` satırı living dispatch'i
  gözlemlenebilir hale getirir.
- `world/entity/Entity.java:2380-2417` ve `world/entity/EntityAttachments.java:47-70`
  aynı zamanda iki satırın position farkını açıklar: Snowball feet fallback'i
  sıfırdır; standing Player vehicle attachment'ı `(0,0.6,0)`'dır.

Oracle fixture'ı gerçek vanilla `Boat`, `Snowball` ve remote `Player`
örneklerini kullanır. Boat'ın tick gövdesi no-op bırakılır. Snowball için
fixture `rideTick` hareket/rotation adımını gerçek projectile state'iyle
çalıştırır; Player fixture'ı gerçek `rideTick` çağrısını ve boat relation'ını
korur. Böylece clamp çağrısı sentetik bir Rust sonucu olarak değil, Java
entity sınıfının gerçek dispatch'i olarak ölçülür.

## Yeni dal ve varyant kararı

Bu kayıt, P08.ba server-conformance kaydındaki boat `clampRotation` dalını
ilk kez Java istemci tick akışıyla `client-parity` olarak çalıştırır. Farklı
gözlem, clamp'in client root/passenger tick sırasından sonra current ve old
yaw'a uygulanması ve living Player'da body/head setter dispatch'inin state'e
yansımasıdır. Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı
**0**'dır.

Tek parametrik corpus kaydı iki Java yolunu temsil eden iki satır kullanır:
base Entity davranışı için Snowball type ID `120` ve living body/head davranışı
için remote Player type ID `156`. Nonzero `deltaRotation`, Breeze tag istisnası
ve Animal kapasite offset'i yeni varyant olarak açılmadı; bunlar ilgili
server-conformance kayıtlarında tutulur.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, konum `(2,2,2)` ve yaw `0.0F` kullanılır.
  Boat ve passenger `NoGravity=true`'dur; world tick'i yalnız rotation ve
  passenger position state'ini ayırmak için bir kez yürütülür.
- Snowball type ID `120`, başlangıçta `(2,12,2)` konumunda, zero movement,
  yaw `170.0F` ve pitch `0.0F` ile boat'a bağlıdır. Java projectile rotation
  adımından sonra `positionRider` clamp'i current yaw'ı `105.0F`'ye, previous
  yaw'ı `139.0F`'ye indirir. Base Entity head/body setter'ları no-op'tur.
- Player type ID `156`, başlangıçta `(2,12,2)` konumunda, zero movement,
  yaw `140.0F` ve pitch `0.0F` ile bağlıdır. `rideTick` sonrası boat seat'i
  `(2,2.1875,2)` olur; Player attachment `(0,0.6,0)` çıkarıldığı için current
  position `(2,1.5875,2)`'dir.
- Player clamp sonucunda current ve previous yaw `105.0F` olur. Current head
  yaw `105.0F`, previous head yaw `140.0F`, current body yaw `0.0F` ve
  previous body yaw `140.0F` olarak saklanır. Bu ayrım, body/head setter
  dispatch'inin ve transform history sırasının birlikte ölçüldüğünü gösterir.
- Her state entity tick count, position, previous position, movement, current /
  previous yaw-pitch ve no-gravity alanlarını taşır. Player satırı buna head ve
  body yaw alanlarını ekler; Snowball satırı yalnız base Entity alanlarını taşır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-rotation` kaydı iki parametrik satırı aynı
initial ve `clamp` state'leri içinde yürütür. Java oracle iki çalıştırmada
kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`1d27a310d73c454537666fb8da8fddd3c6284105571e75cfc694ce49fab7d4da`

Bu capture math ailesinde **3010** case içerir. Seçili Java state'leri:

| Passenger | Current yaw | Previous yaw | Current head/body | Previous head/body |
| --- | ---: | ---: | --- | --- |
| Snowball `120` | `105.0` | `139.0` | base Entity setter no-op | base Entity setter no-op |
| Player `156` | `105.0` | `105.0` | head `105.0`, body `0.0` | head `140.0`, body `140.0` |

Tam corpus Java capture kimliği:
`019f6a89bc5cb73e0976eb055402e73330638133ba9aabc20f551be93fed362c`

Toplam **3871** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::tick` yollarını
kullanır. Passenger type ID'sine göre yalnız Java oracle'da gözlenen Player
body/head alanları seçili state'e eklenir; clamp ve passenger position hesabı
production world kodundan gelir.

Beklenen rotation, history ve position değerleri Java capture'dan alındı;
Rust çıktısından türetilmedi.

Debug adapter SHA-256:
`26731c35729d77636a94de83e4076b38724800ebece095e83021677f6013f9f9`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3010
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3871
difference=null
status=parity_passed
```

Release adapter SHA-256:
`07f500f96c372af7f29ee679561fcf82af75eb518c82cf983768476bc364d176`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3871
difference=null
status=parity_passed
```

`cargo test --locked --offline` sonucu **194 passed**,
`cargo clippy --locked --offline --all-targets -- -D warnings` sonucu temizdir.
`cargo fmt --all -- --check`, Python syntax check ve `git diff --check` de
geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun passenger rotation
packet'lerini kabul etmesi [P08.ba server-conformance kaydında](client-world-entity-passenger-rotation-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Nonzero boat `deltaRotation`, Breeze `CAN_TURN_IN_BOATS` tick istisnası,
Animal kapasite rotation offset'i, `Entity.turn` callback'i, nested controlling
passenger, chest boat/raft rotation, collision, dismount, gerçek oynanabilir
istemci bağlantısı ve render/frame timing'i açık kalır. Bu kayıt P08'in
`client-parity` kapanışı değildir.
