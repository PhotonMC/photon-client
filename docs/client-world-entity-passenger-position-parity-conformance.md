# P08.ax.1 — ClientWorld passenger seat `positionRider` — [client-parity]

Bu kayıt, normal ahşap boat passenger'ının Java istemcisindeki
`positionRider` seat geometry'sinin Rust `ClientWorld` kopyasında eşleşmesini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java
oracle'ına ve production Rust adapter'a verilir.

[P08.ax server-conformance kaydı](client-world-entity-passenger-position-conformance.md)
vanilla sunucusuna bağlı oak boat gözlemlerini tutar. O kayıt
server-conformance kanıtıdır; bu kayıt Java istemci state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `world/entity/Entity.java:2380-2398`, passenger `rideTick` sonrasında
  vehicle'ın `positionRider` yolunu çağırır. Bu yol vehicle passenger riding
  position'ından passenger'ın `VEHICLE` attachment noktasını çıkarır.
- `world/entity/Entity.java:2403-2417` ve
  `world/entity/EntityAttachments.java:47-70`, attachment noktası olmayan
  passenger için feet fallback'inin sıfır olduğunu ve tanımlı noktaların
  vehicle yaw'ıyla döndürüldüğünü belirler. Snowball bu fallback'i kullanır.
- `world/entity/vehicle/boat/AbstractBoat.java:132-150`, bir passenger için
  tekne z-offset'ini `0`, birden fazla passenger için liste sırasına göre
  `0.2F` ve `-0.6F` seçer. `Boat.java:15-17` normal boat yüksekliğini
  `0.5625F / 3.0F` olarak hesaplar.
- `world/entity/vehicle/boat/AbstractBoat.java:631-642,677-684`, gerçek
  `positionRider` çağrısının passenger rotation clamp yan etkisini de
  yürüttüğünü gösterir. Bu slice'ta `deltaRotation` sıfır bırakılmıştır;
  rotation state'i yine karşılaştırma çıktısına dahildir.

## Yeni dal ve varyant kararı

Bu kayıt, P08.ax server-conformance kaydındaki normal boat
`getPassengerAttachmentPoint` dalını ilk kez Java istemcisiyle
`client-parity` olarak çalıştırır. P08.aw.1 generic vehicle ile passenger
`rideTick` sırasını ölçtü; P08.bg generic vehicle attachment fallback'ini
ölçtü. Bu kaydın farklı gözlemi, normal boat'ın tek ve çoklu passenger seat
offset'ini vehicle yaw'ıyla döndürüp current position'a uygulamasıdır. Aynı
sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı, aynı oak boat için önce tek passenger seat'ini,
sonra ilişkiyi iki snowball passenger'a değiştirerek çoklu seat sırasını
çalıştırır. Yalnız sabiti değişen boat family kopyaları açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde oak boat type ID `89`, snowball type ID'leri `120` ve
  vehicle yaw `90.0F` kullanılır.
- Boat `(2,2,2)` konumunda başlar. Snowball `38` başlangıçta tek passenger'dır;
  snowball `39` aynı başlangıç konumunda bağımsız kök entity olarak tutulur.
- İlk world tick'inde boat passenger `38` için tek seat offset'i `0` ve ride
  height `0.1875` uygulanır. Passenger'ın tick başındaki previous position'ı
  korunur; ride tick movement reset'i sonrasında snowball gravity/air drag
  state'i korunur.
- İkinci tick öncesi ordered passenger listesi `[38,39]` yapılır. Boat yaw
  `90` ile döndürülen seat offset'leri sırasıyla `(-0.2,0,0)` ve
  `(0.6,0,0)` olur; current passenger positions sırasıyla
  `(1.8,2.1875,2)` ve `(2.6,2.1875,2)`'dir.
- Bağımsız `39`, ilk tick'te root snowball olarak normal free-flight tick'ini
  alır; relation değişince ikinci tick'te boat passenger `rideTick` yoluna
  girer. Bu, ClientLevel root/passenger seçiminin seat geometry ile aynı
  input'ta korunmasını kontrol eder.

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-position` kaydı initial, single-seat ve
multi-seat state'lerini aynı fixture üzerinde yürütür. Java oracle iki
çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`1365a9a40aa301c25b42b0f626607b1b11162a482ca7b569790cbc887b322acc`

Bu capture math ailesinde **3007** case içerir. Seçili state gözlemleri:

| State | Passenger `38` | Passenger `39` |
| --- | --- | --- |
| `single-seat` | position `(2,2.1875,2)`, previous `(2,12,2)` | bağımsız root olarak free-flight state'i |
| `multi-seat` | position `(1.8,2.1875,2)`, ordered index `0` | position `(2.6,2.1875,2)`, ordered index `1` |

Tam corpus Java capture kimliği:
`03c9f40a36ac49ef2d29ce8acc4cece79ff6eb24551658a243cf13731f70d750`

Toplam **3868** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_passengers` ve
`ClientWorld::tick` yollarını kullanır. Normal boat seat geometry'si,
ordered passenger index'i, yaw dönüşü, previous position history'si ve
passenger tick sırası production world state'inden okunur.

Karşılaştırma sırasında Java `AbstractBoat.clampRotation` ifadesinin f32
işlem sırası da ayrıştırıldı. Java current yaw için `current + targetDelta -
delta` işlemlerini ayrı sırayla yapar; Rust önce `targetDelta - delta`
hesaplayarak bir ara değer kullanıyordu. Bu, seçili state'te Java
`40ccccd0` ile Rust `40cccccd` raw yaw bitleri arasında fark oluşturuyordu.
Rust clamp helper'ı Java'nın kaynak işlem sırasını koruyacak şekilde düzeltildi;
aynı değişiklik Player ve entity passenger yollarına uygulanır.

Debug adapter SHA-256:
`6b4a1d37b99dd6d8564881465e44df24c34dbb0699b786d64f69607200b4b77d`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3007
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3868
difference=null
status=parity_passed
```

Release adapter SHA-256:
`470945ea6123eba5fc60d6aa7827b1a3295746fa2b97a4f23f6a8f046d129790`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3868
difference=null
status=parity_passed
```

İlgili Rust unit testleri, clippy ve runner kontrolleri bu commit öncesinde
ayrıca çalıştırıldı. Vanilla server koşusu [P08.ax
server-conformance kaydında](client-world-entity-passenger-position-conformance.md)
tutulur.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun oak boat passenger
packet'lerini kabul etmesi [P08.ax server-conformance kaydında](client-world-entity-passenger-position-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Player vehicle attachment, chest boat/raft geometry, pose/scale/dimensions'a
bağlı özel attachment'lar, nonzero boat `deltaRotation`, passenger rotation
yan etkilerinin ayrı dalları, nested controlling passenger, collision,
dismount, gerçek oynanabilir istemci bağlantısı ve render/frame timing'i açık
kalır. Bu kayıt P08'in `client-parity` kapanışı değildir.
