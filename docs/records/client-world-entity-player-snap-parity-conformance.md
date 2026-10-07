# P08.as.1 — ClientWorld remote Player large-position snap — [client-parity]

Bu kayıt, remote Player position sync hedefi interpolation eşiğini aştığında
Java istemcisinin snap state'ini Rust `ClientWorld` kopyasıyla karşılaştırır.
Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
production Rust adapter'a verilir.

[P08.as server-conformance kaydı](client-world-entity-player-snap-conformance.md)
vanilla sunucusundan gelen PositionSync akışının Rust world'ünde kabulünü
gösterir. O kayıt wire ve canlı entegrasyon kanıtıdır; bu kayıt Java
istemcisinin snap ve interpolation state'ini ölçer.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientPacketListener.java:639-657`, remote entity
  PositionSync hedefinin mevcut raw position'a squared distance'ını ölçer.
  `4096.0` değerini aşan hedefte `snapTo`, ticking ve eşik altında kalan
  hedefte `moveOrInterpolateTo` seçilir; karşılaştırma strict `>`'dır.
- `world/entity/Entity.java:1808-1814`, `snapTo` current position/body
  rotation ile eski position/rotation history'sini günceller. Bu metodun
  kendi içinde `InterpolationHandler`'ı iptal etmediği source state'te
  gözlendi.
- `world/entity/InterpolationHandler.java:33-78,81-115`, aktif handler'ın
  hedef ve kalan adım state'ini koruduğunu; yeni hedef farklıysa yeniden üç
  adım kurduğunu ve tick başına kalan adıma göre lerp yaptığını tanımlar.
- `client/multiplayer/ClientLevel.java:450-460` ticking entity koşulunu,
  `client/player/RemotePlayer.java:43-62` ise remote Player tick sırasını
  belirler. `util/Mth.java:558-560,608-613` position/yaw/pitch lerp işlem
  sırasını verir.

Oracle, `RemotePlayer` tabanlı fixture ile Java'nın gerçek
`moveOrInterpolateTo` ve `snapTo` metodlarını çalıştırır. Eşik kararının
source'taki ticking koşulu fixture `ClientLevel` üzerinde korunur; profiler,
push ve render yan etkileri bu state için fixture dışında bırakılır.

## Yeni dal ve varyant kararı

Bu dilim, P08.as server-conformance kaydındaki uzaklık üstü Java snap dalını
ilk kez `client-parity` sınıfında çalıştırır. P08.ar.1 küçük hareketin aktif
üç adımlı interpolation dalını ölçer; buradaki farklı gözlem strict eşik
kararı, current/previous history snap'i ve önceden aktif handler'ın Java
`snapTo` sonrasında iptal edilmeden kalmasıdır. Aynı sonucu veren mevcut
client-parity kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı tam `4096.0` sınırını, sınır üstü snap'i, snap
sonrası yeni eşik hareketini ve ikinci snap'i kapsar. Sabit değiştirerek aynı
branch'i çoğaltan ayrı entity kaydı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Player type ID
  `156`'dır. Başlangıç position `(1.0, 80.0, -35.5)`, body yaw/pitch
  `350.0/-20.0` ve head yaw `350.0`'dir. İlk entity snap'i current ve
  previous history'yi eşit başlatır.
- `(1,80,-35.5)` → `(65,80,-35.5)` hareketi tam `64² = 4096.0` sınırındadır.
  Ticking Player için strict `>` koşulu false olur; current değişmeden
  target `65` ve `10/20` rotation ile üç adımlı interpolation başlar.
- İlk tick current position'ı `(22.333333333333336,80,-35.5)` ve kalan adımı
  `2` yapar. Sonraki `(90,80,-35.5)` hedefi mevcut raw position'a göre
  `4096.0` üstündedir; current ve previous position/rotation `90` ve `20/10`
  olur.
- Bu snap sırasında Java `InterpolationHandler` iptal edilmez. Önceki hedef
  `(65,80,-35.5)`, önceki target rotation `10/20` ve kalan `2` adımı aktif
  state'te görünmeye devam eder. Ardından `(154,80,-35.5)` hedefi tam 64
  blok uzaklıkta olduğundan handler yeni hedefle `3` adıma kurulur.
- `(154,80,-35.5)` hedefinin ilk tick'i ölçüldükten sonra `(219,80,-35.5)`
  hedefi tekrar eşik üstü snap olur. Current/previous history yeni hedefe
  giderken daha önce aktif olan handler state'i yine Java semantiğiyle kalır.
- Bu kayıt PositionSync'in remote, ticking ve boş collision-cell koşulunu
  kapsar. Teleport relative helper'ı, local instance authority, collision
  gate'inin reddettiği interpolation, external movement, passenger history,
  render partial-tick state'i ve genel entity tick gövdeleri dışarıdadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-player-snap` kaydı
strict boundary, birinci interpolation tick'i, iki snap ve snap arası yeni
boundary hedefini ardışık olarak taşır. Java oracle iki çalıştırmada kararlı
kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`da4e261614dbba47bf0d33cd010b007523495b39ef80f38d4319100f2c5103eb`

Bu kayıt math ailesinde **3002** case içerir. P08.as state özeti:

| Adım | current position | current yaw/pitch | interpolation state |
| --- | --- | --- | --- |
| initial | `(1,80,-35.5)` | `350/-20` | inactive, 0 |
| exact-threshold-interpolates | başlangıç | `350/-20` | target `(65,80,-35.5)`, `10/20`, 3 |
| exact-threshold-first-tick | `22.333333333333336,80,-35.5` | `356.66666/-6.6666665` | aynı target, 2 |
| over-threshold-snaps | `(90,80,-35.5)` | `20/10` | eski target `(65,80,-35.5)`, `10/20`, 2 |
| exact-threshold-after-snap | `(90,80,-35.5)` | `20/10` | target `(154,80,-35.5)`, `30/5`, 3 |
| boundary-first-tick | `111.33333333333333,80,-35.5` | `23.333334/8.333333` | aynı target, 2 |
| second-over-threshold-snaps | `(219,80,-35.5)` | `40/-5` | eski target `(154,80,-35.5)`, `30/5`, 2 |

Snap history'si ile handler state'i ayrı gözlenir: snap current/previous
transformu hedefe eşitler, aktif handler'ın internal target ve steps değeri
Java'da ayrıca kalır. Karşılaştırma double veya float epsilon kullanmaz; raw
bit state'i capture içinde taşır.

Tam corpus Java capture kimliği
`a8322a87a0284a311d1dcc935ef1afbfb8e66d00ecdaee0ecb2819e87a816566` ve toplam
**3863** case'tir.

## Rust doğrulaması

Adapter production `ClientWorld::apply_entity`,
`ClientWorld::apply_entity_position_sync`, `ClientWorld::tick` ve
`ClientEntity` interpolation/history getter'larını kullanır. PositionSync
wrapper'ı Java eşik dalını mevcut production position application path'ine
verir. Snap history current/previous transformu hedefe eşitlerken önceden
aktif Player position interpolation state'ini korur; yeni hedef geldiğinde
aynı handler state'i production interpolation yolunda güncellenir.

Debug adapter SHA-256:
`bc8507cdc627f903059deacc2ee736ec6e2e579088ae829e370a67a8b09684fc`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3002
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3863
difference=null
status=parity_passed
```

Release adapter SHA-256:
`69c3959ab59e9f139e0bd872f2b6c18b9f31915a910481ee8298d2e9783032fc`

Release adapter aynı full capture üzerinde **3863/3863** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3863
difference=null
status=parity_passed
```

Java full corpus tekrar capture'ı iki tekrar ve `difference=null` ile
`reference_ready` oldu. Full compare binary/math/nbt/resources/timer/utf8/wire
ailesindeki **3863/3863** kaydı geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java istemci
oracle'ına ve Rust adapter'a verilir. Vanilla sunucusuna canlı bağlantı ve
wire kabulü [P08.as server-conformance kaydında](client-world-entity-player-snap-conformance.md)
tutulur; canlı sunucu sonucu Java snap/interpolation state karşılaştırmasının
yerine geçmez.

## Açık kapsam

Teleport packet'inin snap/history helper'ı, local authority ve passenger
konumlandırması, collision sonucu hareket düzeltmesi, external movement
delta'sı, generic LivingEntity türleri, gerçek ClientPacketListener dispatch'i,
render/frame timing'i, motion integration'i ve oynanabilir istemci bağlantısı
açık kalır.
