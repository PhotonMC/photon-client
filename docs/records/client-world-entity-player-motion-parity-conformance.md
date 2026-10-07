# P08.at.1 — ClientWorld remote Player motion interpolation — [client-parity]

Bu kayıt, remote Player'a gelen motion hedefinin Java istemcisindeki üç adımlı
interpolation state'i ile Rust `ClientWorld` kopyasında eşleşmesini karşılaştırır.
Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve
production Rust adapter'a verilir.

[P08.at server-conformance kaydı](client-world-entity-player-motion-conformance.md)
vanilla sunucusunun knockback sonucu gönderdiği motion packet'inin Rust
world'ünde kabulünü gösterir. O kayıt canlı wire ve entegrasyon kanıtıdır; bu
kayıt Java remote Player motion state'ini aynı inputta ölçer.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientPacketListener.java:621-627`, mevcut entity için
  `lerpMotion` çağrısını seçer; packet uygulanırken entity'nin mevcut motion'ı
  ayrıca değiştirilmez.
- `world/entity/Entity.java:2644-2646`, genel entity yolunda `lerpMotion`
  çağrısını doğrudan delta movement replacement'a yönlendirir.
- `client/player/RemotePlayer.java:12,43-62,72-76`, remote Player için packet
  hedefini ayrı `lerpDeltaMovement` state'inde tutar. Her `aiStep` motion
  adımında hedef-current farkını kalan adım sayısına böler, mevcut motion'a
  ekler ve sayacı azaltır.
- `world/entity/EntityTypes.java:1163-1172`, Player update interval'ını `2`
  yapar. Remote Player override'ı bu değere bir ekleyerek üç adımlı state
  kurar.

Oracle, gerçek Java `RemotePlayer.lerpMotion` ve `Entity.addDeltaMovement`
metotlarını kullanır. `RemotePlayer.aiStep` içindeki motion dalı kaynak sırası
ve aynı double arithmetic ile kontrollü fixture'da yürütülür; minimal oracle
classpath'inde bulunmayan `jtracy` profiler ve push/render yan etkileri bu
state'in dışında bırakılır.

## Yeni dal ve varyant kararı

Bu kayıt, P08.at server-conformance kaydındaki remote Player motion
interpolation dalını ilk kez `client-parity` sınıfında çalıştırır. P08.ar.1
position interpolation, P08.as.1 ise uzaklık üstü position snap state'ini
ölçer; buradaki farklı gözlem motion hedefinin packet anında current değeri
koruması, üç adımlı fark bölümü ve aktif interpolation sırasında yeni hedefin
üç adıma yeniden kurulmasıdır. Aynı sonucu veren mevcut client-parity
kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı başlangıç motion'ını, ilk hedefi, iki ara tick'i,
aktif hedef replacement'ını ve replacement tamamlanmasını kapsar. Yalnızca
sabit değiştirerek aynı branch'i çoğaltan ayrı entity kaydı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Player type ID
  `156`'dır. Başlangıç position `(1.0,80.0,-35.5)`, body yaw/pitch
  `350.0/-20.0`, head yaw `350.0` ve current motion
  `(0.125,0.25,-0.375)`'tir.
- Motion hedefi `(0.875,-0.5,0.625)` packet uygulamasında ayrı target state'e
  yazılır; current motion başlangıç değerinde kalır ve kalan adım `3` olur.
- İlk iki tick'te hedef-current farkı sırasıyla `3` ve `2` kalan adıma
  bölünerek mevcut motion'a eklenir. Üçüncü adımda hedefe ulaşılır ve kalan
  adım `0` olur.
- İkinci tick sonrasında yeni hedef `(-0.375,0.75,1.125)` gelirse current
  motion korunur, target değişir ve kalan adım yeniden `3` olur. Sonraki bir
  tick ve iki tick'lik completion state'i aynı formülle karşılaştırılır.
- Bu kayıt Player'ın motion interpolation dalını boş, çarpışmasız bir client
  world'de ölçer. Generic entity raw motion replacement'ı, yerçekimi,
  collision, push, position/rotation interpolation, passenger state,
  teleport delta movement ve render partial tick kapsam dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-player-motion` kaydı, packet anı ve ardışık world tick
state'lerini taşır. Java oracle iki çalıştırmada kararlı kaldı; tekrar farkı
`null`'dır.

Math capture kimliği:
`f5089920a2d92b58671ec67dbb3daa0efb52c0b0a7ae81f07acc012a112a38ec`

Bu kayıt math ailesinde **3003** case içerir. Yeni kaydın seçili raw state
bitleri:

| Adım | current motion X/Y/Z | hedef | kalan adım |
| --- | --- | --- | ---: |
| initial | `3fc0000000000000/3fd0000000000000/bfd8000000000000` | yok | 0 |
| packet-target-keeps-current-motion | aynı | `3fec000000000000/bfe0000000000000/3fe4000000000000` | 3 |
| first-motion-tick | `3fd8000000000000/0000000000000000/bfa5555555555558` | aynı | 2 |
| second-motion-tick | `3fe4000000000000/bfd0000000000000/3fd2aaaaaaaaaaab` | aynı | 1 |
| replacement-target-resets-three-steps | ikinci state | `bfd8000000000000/3fe8000000000000/3ff2000000000000` | 3 |
| replacement-first-tick | `3fd2aaaaaaaaaaab/3fb5555555555554/3fe238e38e38e38e` | aynı | 2 |
| replacement-final-two-ticks | `bfd8000000000000/3fe8000000000000/3ff2000000000000` | yok | 0 |

Karşılaştırma double veya float epsilon kullanmaz; motion ve target raw bit
state'i capture içinde taşır. Tam corpus Java capture kimliği
`0683f01655413a7cda48db1a40b95235c7db08c850600c2667cde7127d8c0918` ve toplam
**3864** case'tir.

## Rust doğrulaması

Adapter production `ClientWorld::apply_entity`,
`ClientWorld::apply_entity_motion`, `ClientWorld::tick` ve
`ClientEntity` motion state getter'larını kullanır. Başlangıç motion'ı
`EntityInput` ile kurulur; remote Player motion packet'i
`EntityMotionInput` üzerinden hedef state'e alınır. World tick, position/head
adımlarından sonra Player motion interpolation'ını yürütür.

Debug adapter SHA-256:
`12fc03d5d23d12af64c131f45b3c6b7b8b485e260a99788e1991e0294925f21a`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3003
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3864
difference=null
status=parity_passed
```

Release adapter SHA-256:
`ed75229de0f3e353c2736dc2eb3cdd0f419e19e221a503ac8bc47ae03a28acae`

Release adapter aynı full capture üzerinde **3864/3864** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3864
difference=null
status=parity_passed
```

Java full corpus tekrar capture'ı iki tekrar ve `difference=null` ile
`reference_ready` oldu. Full compare binary/math/nbt/resources/timer/utf8/wire
ailesindeki **3864/3864** kaydı geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java istemci
oracle'ına ve Rust adapter'a verilir. Vanilla sunucusu üzerindeki canlı motion
packet ve knockback kabulü [P08.at server-conformance kaydında](client-world-entity-player-motion-conformance.md)
tutulur; canlı sunucu sonucu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Generic LivingEntity ve diğer entity türlerinin gerçek client tick/fizik,
yerçekimi, collision ve push davranışı; hızlı ardışık motion packet'leri,
position/rotation ile motion birleşik tick sırası, passenger ilişkisi,
teleport delta/relative motion, local Player authority, render/frame timing'i
ve oynanabilir istemci bağlantısı açık kalır. Bu kayıt P08'in
`client-parity` kapanışı değildir.
