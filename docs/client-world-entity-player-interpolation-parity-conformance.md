# P08.ar.1 — ClientWorld remote Player position interpolation — [client-parity]

Bu kayıt, remote Player için küçük position/rotation hareket paketinden sonra
kurulan üç adımlı interpolation state'inin Java Edition 26.2 ve Rust
`ClientWorld` kopyasında eşleşmesini kapsar. Kanıt sınıfı `client-parity`'dir:
aynı özgün sentetik input Java istemci oracle'ına ve production Rust adapter'a
verilir.

[P08.ar server-conformance kaydı](client-world-entity-player-interpolation-conformance.md)
vanilla sunucusundan gelen canlı wire akışının Rust world'ünde kabulünü
gösterir. O kayıt protokol ve entegrasyon kanıtıdır; bu kayıt Java istemcisiyle
aynı state karşılaştırmasını yapar.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `world/entity/InterpolationHandler.java:10-115`, hedef position/yaw/pitch
  state'ini, kalan adımları ve önceki interpolation tick'inin transformunu
  saklar. Varsayılan uzunluk `3`'tür; her adımda kalan sayının tersi alpha
  olarak kullanılır.
- Aynı handler position için double `Mth.lerp`, yaw için wrapped
  `Mth.rotLerp`, pitch için linear lerp uygular. Aktif hedef değişirse
  previous transform current değerden yeniden kurulur; hedef ve rotation aynı
  kaldığında sayaç yeniden başlatılmaz.
- `world/entity/LivingEntity.java:252,3042-3056,3306-3315` handler'ın living
  entity state'ine bağlı olduğunu ve tick içindeki interpolation sırasını
  gösterir. `client/player/RemotePlayer.java:43-62`, remote Player `aiStep`
  dalında position interpolation'ın head/motion adımlarından önce çağrıldığını
  tanımlar.
- `world/entity/Entity.java:2535-2565`, `moveOrInterpolateTo` çağrısının
  handler hedefi kurduğunu; `client/multiplayer/ClientPacketListener.java:639-754,806-823`
  ise MoveEntity/PositionSync/Teleport paketlerinin ticking ve uzaklık
  koşullarına göre interpolation veya snap seçtiğini tanımlar.
- `world/entity/Entity.java:1816-1844,1960-1964,2000-2004` ilk snap'in
  position ve rotation geçmişini current değerle başlatmasını ve render
  history sorgularını; `util/Mth.java:558-560,608-613` lerp işlem sırasını
  belirler.

Oracle, `RemotePlayer` tabanlı fixture ile gerçek `moveOrInterpolateTo` ve
`InterpolationHandler.interpolate` metodlarını çalıştırır. `RemotePlayer.aiStep`
dalındaki interpolation koşulu korunur; bu state için gerekmeyen profiler ve
push yan etkileri minimal oracle classpath'inde fixture dışı bırakılır.

## Yeni dal ve varyant kararı

Bu dilim, P08.ar server-conformance kaydındaki Java remote Player
`InterpolationHandler` dalını ilk kez `client-parity` sınıfında çalıştırır.
P08.u.1 generic transform-history kaydı yalnız world tick snapshot'ını ölçer;
buradaki farklı gözlem aktif network interpolation hedefi, `1/3 → 1/2 → 1`
alpha sırası, aynı hedefte kalan sayacın korunması ve değişen hedefte üç adıma
yeniden başlanmasıdır. Aynı sonucu veren mevcut client-parity kayıtlarının
sayısı **0**'dır.

Sabit değiştirerek aynı branch'i çoğaltan ayrı entity veya angle kaydı açılmadı.
Tek parametrik corpus kaydı wrapped yaw, linear pitch, hedef replacement,
packet short delta çözümü ve tamamlanma durumunu ardışık işlemlerle kapsar.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Player type ID
  `156`'dır. Başlangıç position `(1.0, 80.0, -35.5)`, body yaw/pitch
  `350.0/-20.0` ve head yaw `350.0`'dir. İlk entity snap'i current ve
  previous position/rotation'ı eşit başlatır.
- Position içeren MoveEntity girdisi, `(4.0, 82.0, -34.5)` hedefini ve
  `10.0/20.0` body rotation'ını kurar. Current transform packet anında
  değişmez; aktif adım sayısı `3` olur.
- Birinci ve ikinci world tick sırasıyla `alpha=1/3` ve `alpha=1/2` kullanır;
  üçüncü tick hedefe ulaşır. Yaw wrapped, pitch linear hesaplanır ve raw
  float bitleri karşılaştırmada korunur.
- Aynı hedef ve rotation interpolation tamamlanmadan tekrar gelirse current,
  target ve kalan adım state'i korunur. Hedef `(5.0, 81.0, -33.5)` ve
  `190.0/-10.0` olarak değiştiğinde interpolation current state'ten yeniden
  `3` adımla başlar.
- Bu kayıt boş, çarpışmasız bir client world'de external movement olmadan
  çalışan position branch'ini kapsar. Collision gate'i, uzaklık üstü snap,
  relative/teleport özel dalları, motion interpolation, genel entity tick ve
  renderer bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/entity-player-interpolation` kaydı başlangıç snap'ini, iki
interpolation tick'ini, aynı hedef packet'ini, hedef replacement'ı ve son iki
completion tick'ini kapsar. Java oracle iki çalıştırmada kararlı kaldı; tekrar
farkı `null`'dır.

Math capture kimliği:
`b4f62dc988990b8d68bbd3ba5b1f8ac48a3a18d95a7bd87cf5f0efdf71c6e654`

Bu kayıt math ailesinde **3001** case içerir. P08.ar gözleminin önemli raw
state'leri:

| Adım | current position | current yaw/pitch | interpolation target | kalan adım |
| --- | --- | --- | --- | ---: |
| initial | `(1, 80, -35.5)` | `350/-20` | yok | 0 |
| packet-target-starts-interpolation | başlangıç | `350/-20` | `(4, 82, -34.5)`, `10/20` | 3 |
| first-interpolation-tick | `2, 80.66666666666667, -35.166666666666664` | `356.66666/-6.6666665` | aynı | 2 |
| second-interpolation-tick | `3, 81.33333333333334, -34.83333333333333` | `3.333313/6.666667` | aynı | 1 |
| changed-target-restarts-interpolation | ikinci state | `3.333313/6.666667` | `(5, 81, -33.5)`, `190/-10` | 3 |
| replacement-final-two-ticks | `(5, 81, -33.5)` | `190/-10` | yok | 0 |

Karşılaştırma double veya float epsilon kullanmaz; capture raw bit state'i
taşır. Tam corpus Java capture kimliği
`ba6a495abfa43b16ffcae2e36c6181fc25bd988998638412ee52488b375e1766` ve toplam
**3862** case'tir.

## Rust doğrulaması

Adapter production `ClientWorld::apply_entity`, `apply_entity_move`,
`ClientWorld::tick` ve `ClientEntity` interpolation state getter'larını
kullanır. `apply_entity_move`, protocol short delta'sını Java quantized base
üzerinden çözer; Player type ID `156` için mevcut production interpolation
path'ine target position/rotation verir. World tick position interpolation'ı
head ve motion state'lerinden önce yürütür.

Debug adapter SHA-256:
`a9660c9601b1aee52f9994aed2bc1fd3018140903966774ed4b7816461f9ad64`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3001
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3862
difference=null
status=parity_passed
```

Release adapter SHA-256:
`3b7fd8bb185c98f6bef3dc59a3e281b2077b78637a9a87742ae0029a0f43e13b`

Release adapter aynı full capture üzerinde **3862/3862** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3862
difference=null
status=parity_passed
```

Java referansının tam corpus tekrar capture'ı da `difference=null` ile
`reference_ready` oldu. Full compare, binary/math/nbt/resources/timer/utf8/wire
ailesindeki **3862/3862** kaydı geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java istemci
oracle'ına ve Rust adapter'a verilir. Vanilla sunucusuna canlı bağlantı ve
wire kabulü [P08.ar server-conformance kaydında](client-world-entity-player-interpolation-conformance.md)
tutulur; canlı sunucu sonucu bu Java istemci state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Collision gate ve external movement delta'sı, uzaklık eşiği üstü snap ve
history düzeltmesi, relative movement/teleport packet dalları, remote Player
motion interpolation, genel LivingEntity ve entity tick gövdeleri,
passenger/fizik, render partial-tick state'i, gerçek frame/thread dispatch'i
ve oynanabilir istemci bağlantısı açık kalır.
