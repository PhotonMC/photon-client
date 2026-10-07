# P08.au.1 — ClientWorld remote entity teleport delta and relative motion — [client-parity]

Bu kayıt, `ClientboundTeleportEntityPacket` içindeki relative position,
rotation, delta movement ve `ROTATE_DELTA` çözümünün Java istemcisiyle Rust
`ClientWorld` kopyasında eşleşmesini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.au server-conformance kaydı](client-world-entity-teleport-delta-conformance.md)
vanilla sunucusundan gelen teleport packet'inin wire ve canlı world uygulama
akışını gösterir. O kayıt sunucu uyum kanıtıdır; bu kayıt Java istemci
history/interpolation ve motion state'ini aynı inputta ölçer.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientPacketListener.java:666-698`, teleport entity
  paketinde relative position bitleri varsa interpolation koşulunu açar ve
  `setValuesFromPositionPacket` çağrısına gider. Bu koşu remote, ticking
  Player fixture'ı kullandığı için `interpolate=true`'dir.
- `client/multiplayer/ClientPacketListener.java:806-826`, önce mevcut
  `PositionMoveRotation` state'ini, sonra relative maskeyle absolute target'ı
  hesaplar. Squared distance `4096.0` değerini aşmıyorsa
  `moveOrInterpolateTo` ve yeni delta movement uygulanır; aksi halde raw
  position/rotation/motion yazılır ve eski history ayrı bir
  `PositionMoveRotation` hesabıyla güncellenir.
- `world/entity/PositionMoveRotation.java:40-67`, X/Y/Z ve Y/X rotation
  relative bitlerini çözer, X rotation'ı `[-90,90]` aralığına clamp eder ve
  `ROTATE_DELTA` için mevcut motion'ı önce X sonra Y ekseninde döndürür.
  `world/entity/Relative.java:10-19,50-114` bit numaralarını verir.
- `world/phys/Vec3.java:232-247`, X/Y motion dönüşlerinin double çarpma ve
  toplama sırasını; `world/entity/Entity.java:1816-1844,2535-2565` ise raw
  position/history ve interpolation handler girişini belirler.

Oracle, gerçek Java `PositionMoveRotation.calculateAbsolute`,
`RemotePlayer.moveOrInterpolateTo`, `setDeltaMovement` ve history setter'larını
kullanır. Private `setValuesFromPositionPacket` akışı aynı source-order ile
fixture'da ifade edilir. Position interpolation tick'i gerçek
`InterpolationHandler.interpolate` ile çalıştırılır; minimal oracle
classpath'inde bulunmayan profiler/push/render yan etkileri state dışında
bırakılır.

## Yeni dal ve varyant kararı

Bu kayıt, P08.au server-conformance kaydındaki teleport delta/relative motion
dalını ilk kez `client-parity` sınıfında çalıştırır. P08.ar.1 ve P08.as.1
remote Player position packet'lerinin ayrı interpolation/snap dallarını
ölçer; buradaki farklı gözlem teleport'un dokuz relative bitini tek
`PositionMoveRotation` hesabında birleştirmesi, `ROTATE_DELTA` sırası, büyük
teleport history yazımı ve snap sonrasında aktif handler hedefinin sürmesidir.
Aynı sonucu veren mevcut client-parity kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı tüm relative maskeyi, küçük teleport
interpolation'ını, bir tick sonrasındaki absolute uzak snap'i ve snap sonrası
relative hedef güncellemesini kapsar. Sabit değiştirerek aynı branch'i
çoğaltan ayrı entity kaydı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve remote Player type ID
  `156`'dır. Başlangıç position `(1.0,80.0,-35.5)`, body yaw/pitch
  `30.0/-20.0`, head yaw `40.0` ve motion `(1.0,2.0,3.0)`'tür.
- İlk packet `relative_flags=511` ile position `(2,3,4)`, rotation `15/10`
  ve delta motion `(0.25,-0.5,0.75)` taşır. Absolute sonuç position
  `(3,83,-31.5)`, rotation `45/-10` olur; mevcut motion önce X sonra Y
  döndürülür, ardından relative delta eklenir. Position interpolation state'i
  üç adımda kurulur; motion ise packet sonucuyla hemen değişir.
- İlk world tick'i position ve rotation history'sini snapshot'lar, sonra
  interpolation handler'ı `1/3` adımı uygular. Teleport motion interpolation'a
  girmez; `setDeltaMovement` sonucu current motion olarak kalır.
- İkinci packet absolute position `(100,80,-35.5)`, rotation `-170/80` ve
  motion `(0.5,0.25,-0.75)` taşır. Distance eşiği aşıldığı için current raw
  transform snap edilir; Java'nın set-values snap dalı history'yi ayrı
  hesaplar ve daha önce aktif interpolation handler'ı iptal etmez.
- Üçüncü packet `relative_flags=255` ile position `(1,1,1)`, rotation `5/-5`
  ve delta motion `(-0.125,0.25,0.5)` taşır. Aktif handler target'ı üzerinden
  yeni position `(4,84,-30.5)` ve rotation `50/-15` kurulur; current raw
  position snap sonrasındaki `(100,80,-35.5)` değerinde kalır.
- Bu kayıt remote, ticking ve boş collision-cell koşulundaki teleport state'ini
  kapsar. Local authority/passenger fallback'i, removed vehicle ID'si,
  collision gate'i, generic entity tick/fizik, on-ground yan etkisi, packet
  codec'i ve render partial tick kapsam dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-teleport` kaydı tüm relative maskeyi, bir tick sonrası
uzak snap'i, snap sonrası relative hedefi ve son interpolation tick'ini taşır.
Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`9e4d382357d961e5e38f414610fb59c3e6fd402f3521f700d18089969e15c9cc`

Bu kayıt math ailesinde **3004** case içerir. P08.au state özeti:

| Adım | current position | interpolation target | current motion |
| --- | --- | --- | --- |
| initial | `(1,80,-35.5)` | yok | `(1,2,3)` |
| relative-all-interpolates | başlangıç | `(3,83,-31.5)`, `45/-10`, 3 | `3fd7247e6722ec40/3fee5c6460000000/4010ca961b89a2cc` |
| relative-first-tick | `1.6666666666666665,81,-34.166666666666664` | aynı, 2 | aynı |
| absolute-over-threshold-snaps | `(100,80,-35.5)` | eski target, 2 | `(0.5,0.25,-0.75)` |
| relative-after-snap | `(100,80,-35.5)` | `(4,84,-30.5)`, `50/-15`, 3 | `(0.375,0.5,-0.25)` |
| relative-after-snap-first-tick | `68,81.33333333333333,-33.16666666666667` | aynı, 2 | aynı |

Snap state'i ile handler state'i ayrı karşılaştırılır. Position, rotation,
motion ve history raw bitleri capture içinde taşınır; double veya float
epsilon kullanılmaz. Tam corpus Java capture kimliği
`97097b595f80f27024b032d36bcfc6c91f3b41af997021c72c5d788b54a962d9` ve toplam
**3865** case'tir.

## Rust doğrulaması

Adapter production `ClientWorld::apply_entity`,
`ClientWorld::apply_entity_teleport`, `ClientWorld::tick` ve
`ClientEntity` interpolation/history/motion getter'larını kullanır.
`EntityTeleportInput` relative maskeyi ve delta movement'ı production teleport
path'ine taşır; world tick position interpolation'ını yürütür.

Debug adapter SHA-256:
`768b0424bce83d89ab62ff45a7cb13cd923a4b64ee29382959e763c8c5841b1d`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3004
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3865
difference=null
status=parity_passed
```

Release adapter SHA-256:
`192f10883a28ca078be18bb940be3ead162ee72dd785b180d27bc5799cdee18b`

Release adapter aynı full capture üzerinde **3865/3865** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3865
difference=null
status=parity_passed
```

Java full corpus tekrar capture'ı iki tekrar ve `difference=null` ile
`reference_ready` oldu. Full compare binary/math/nbt/resources/timer/utf8/wire
ailesindeki **3865/3865** kaydı geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java istemci
oracle'ına ve Rust adapter'a verilir. Vanilla sunucusuna canlı teleport ve
relative packet kabulü [P08.au server-conformance kaydında](client-world-entity-teleport-delta-conformance.md)
tutulur; canlı sunucu sonucu Java client state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Local Player authority ve removed vehicle fallback'i, passenger yeniden
konumlandırması, generic entity interpolation/fizik, collision-aware movement,
on-ground transition, hızlı ardışık teleport/codec base'i, render/frame timing'i
ve oynanabilir istemci bağlantısı açık kalır. Bu kayıt P08'in
`client-parity` kapanışı değildir.
