# P08.u.1 — ClientWorld entity transform geçmişi ve interpolation — [client-parity]

Bu kayıt, local Player ve ticking remote entity için transform geçmişinin
world tick sınırında tutulmasını ve position/body/head interpolation
sorgularını Java Edition 26.2 ile aynı özgün sentetik input üzerinde
karşılaştırır. Kanıt sınıfı `client-parity`'dir: Java oracle ve production
Rust adapter aynı girdiyi alır.

[P08.u server-conformance kaydı](client-world-entity-transform-conformance.md)
vanilla sunucusundan alınan transform state'inin Rust world'ünde kabulünü
gösterir. O kayıt wire ve canlı entegrasyon kanıtıdır; bu kayıt Java
istemcisiyle state eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientLevel.java:450-477`, ticking root entity için
  `setOldPosAndRot()` çağrısının tick gövdesi ve `tickCount` artışından önce
  geldiğini gösterir. Local Player always-ticking olsa da aynı transform
  snapshot sırasını izler.
- `world/entity/Entity.java:1816-1844`, position ve body rotation geçmişini
  current değerlerden doldurur; `snapTo` yeni entity'nin ilk geçmişini current
  transform ile başlatır.
- `world/entity/Entity.java:1960-1964,2000-2004`, position için double
  `Mth.lerp`, yaw için wrapped `Mth.rotLerp`, pitch için linear lerp ve
  `partialTick == 1.0F` için doğrudan current rotation yolunu tanımlar.
- `world/entity/LivingEntity.java:240-241,498,3357`, living head yaw için
  `yHeadRotO` geçmişini ve wrapped interpolation yolunu ekler.
- `util/Mth.java:558-560,608-613`, lerp işlemlerinin source order'ını
  belirler. `client/multiplayer/ClientPacketListener.java:806-824` ilk
  oyuncu position snap'inde previous state'in hedef transforma alınmasını
  gösterir.

Bu slice, network paketlerinin genel `moveOrInterpolateTo` hedef geçmişini
modellemez. Kontrollü fixture, önce ilk snap'i kurar; bir world tick'i ile
history'yi ilerletir; sonra current transform güncellemesini bir sonraki tick
öncesinde sorgular ve son tick'in yeni history snapshot'ını ölçer.

## Yeni dal ve varyant kararı

Bu dilim, P08.u server-conformance kaydındaki transform geçmişi dalını ilk
kez `client-parity` sınıfında çalıştırır. Farklı gözlem; ilk local Player snap
sonrası previous/current eşitliği, tick sonrası yapılan position ve rotation
güncellemesinin previous state'i koruması, `350° → 10°` yaw/head sarmalaması,
pitch'in linear interpolation'ı ve `partialTick == 1.0F` doğrudan current
sonucudur. Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı
**0**'dır; P08.u server-conformance ve world unit testi Java istemcisiyle
aynı capture karşılaştırması yapmaz.

İlk snap, ara update, `partialTick=1.0F` sorgusu ve sonraki tick tek
parametrik corpus kaydında ardışık işlemlerle kapsandı. Sabit değiştirerek
aynı branch'i çoğaltan entity veya angle varyantı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Başlangıç
  dimension min Y `0` olan ve remote entity'nin bulunduğu chunk `(0,-3)`'ün
  tek boş section'ı yüklüdür.
- Local Player ID `42`, `(10,64,-2)` konumunda `y=350°`, `x=-20°` rotation'ı
  ile başlar. İlk position snap'i sonrasında previous position ve previous
  rotation current değerlerle aynıdır.
- Remote Zombie entity ID `37`, type ID `151`,
  `(1.25,80,-35.5)` konumunda `y=350°`, `x=-20°`, head `350°` ile başlar.
  Yeni entity snap'i position, body rotation ve head rotation geçmişini
  current değerle başlatır.
- İlk world tick'inde game time ve iki entity'nin tick sayacı birer artar;
  current transform değişmediği için previous değerler aynı kalır.
- Sonraki update local Player'ı `(14,68,6)`, `y=10°`, `x=20°` değerlerine;
  remote entity position'ını `(3,82,-34.5)` ve head yaw'ını `10°` değerine
  taşır. Tick henüz çalışmadığı için previous state ilk tick snapshot'ında
  kalır.
- `partialTick=0.5F` için local Player position `(12,66,2)`, remote
  position `(2.125,81,-35)` ve wrapped yaw/head `360°` gözlenir. Pitch
  `0°` olur. `partialTick=1.0F` current rotation ve current position'ı
  doğrudan döndürür.
- Sonraki world tick'i update edilmiş current transformu previous state'e
  alır ve tick sayısını/game time'ı tekrar bir artırır. Entity tick gövdeleri,
  network interpolation target/step state'i, teleport snap eşikleri,
  passenger transformları, physics, render frame zamanlaması ve oynanabilir
  istemci bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/entity-transform-history` kaydı ilk snap, tick, transform
update, iki partial-tick sorgusu ve son tick'i içerir. Java oracle iki
çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Java referans kimliği:
`1fe3e1b322e3b91a2d04a5a3040a2206311d94acfb662eefb35e15861125c017`

Math capture'ı **3000** kayıttır. Transform state özeti:

| Adım | game time | local tick | remote tick | local interpolated position | remote interpolated position | wrapped yaw/head |
| --- | ---: | ---: | ---: | --- | --- | --- |
| initial | 0 | 0 | 0 | `(10,64,-2)` | `(1.25,80,-35.5)` | `350° / 350°` |
| snapshot-initial | 1 | 1 | 1 | `(10,64,-2)` | `(1.25,80,-35.5)` | `350° / 350°` |
| updated-transform, `0.5` | 1 | 1 | 1 | `(12,66,2)` | `(2.125,81,-35)` | `360° / 360°` |
| partial-one, `1.0` | 1 | 1 | 1 | `(14,68,6)` | `(3,82,-34.5)` | `10° / 10°` |
| snapshot-updated | 2 | 2 | 2 | `(14,68,6)` | `(3,82,-34.5)` | `10° / 10°` |

Raw float/double bit çıktıları capture içinde tutulur; karşılaştırma
epsilon kullanmaz.

## Rust doğrulaması

Adapter production `ClientWorld::set_local_player_id`, initial position
application, `apply_entity`, `ClientWorld::tick`,
`update_entity_position`, `apply_entity_head_rotation` ve interpolation
getter'larını kullanır. İlk local Player position uygulaması, Java'nın ilk
position snap'inde yaptığı gibi tick başlamadan önce previous position/body
rotation history'sini current değere kurar. Bu davranış
`tick_captures_previous_transform_for_position_and_rotation_interpolation`
unit testinde de kontrol edilir.

Debug adapter SHA-256:
`2f59be39ccb33a3be1fbd43eced6a8c07b30e852d596e0686065191c87c1eea6`

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3000
difference=null
status=parity_passed
```

Java capture tekrarı:

```text
python3 tools/behavior/run.py verify --family math
case_count=3000
difference=null
status=reference_ready
```

Release adapter SHA-256:
`091de9f485849a332c021de2cef6b660292fad2b73dc3887a1da75c286abb11c`

Release adapter aynı capture üzerinde **3000/3000** kayıt ve
`difference=null` ile `parity_passed` oldu.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/release/examples/behavior_adapter
case_count=3000
difference=null
status=parity_passed
```

Ek kontroller:

```text
cargo test --locked --offline --all-targets --quiet
194 passed; 0 failed
cargo clippy --locked --offline --all-targets -- -D warnings
finished successfully
python3 -m unittest discover -s tools/behavior -p 'test_*.py'
5 tests; OK
```

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün sentetik input Java istemci
oracle'ına ve Rust adapter'a verilir. Vanilla sunucusuna gerçek bağlantı ve
wire kabulü [P08.u server-conformance kaydında](client-world-entity-transform-conformance.md)
tutulur; canlı sunucu sonucu bu Java state karşılaştırmasının yerine
geçmez.

## Açık kapsam

Genel `moveOrInterpolateTo` çok adımlı network interpolation'ı, teleport
snap'in uzaklık eşiği ve history düzeltmesi, relative movement packet'leri,
gerçek entity tick/motion gövdeleri, frozen/removed/passenger filtreleri,
LivingEntity body rotation yan etkileri, collision, physics, renderer,
frame/partial-tick zamanlaması ve oynanabilir istemci bağlantısı açık kalır.
