# P08.ap.1 — ClientWorld remote Player head rotation — [client-parity]

Bu kayıt, `ClientboundRotateHeadPacket` sonrası remote Player entity'sinde
kurulan üç adımlı head rotation interpolation state'inin Java 26.2 ve Rust
`ClientWorld` kopyasında eşleşmesini kapsar. Kanıt sınıfı `client-parity`'dir:
aynı özgün sentetik input Java oracle'ına ve production Rust adapter'a verilir.

Önceki [P08.ap server-conformance kaydı](client-world-entity-player-head-rotation-conformance.md)
Mojang vanilla 26.2 dedicated server'ın remote Player için gönderdiği
`RotateHead` akışını kabul ettiğini gösterir. O kayıt wire ve canlı sunucu
kanıtıdır; bu kayıt istemci iç state'ini Java ile karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `network/protocol/game/ClientboundRotateHeadPacket.java:12-50` entity ID
  ve packed head byte'ını çözer; `Mth.unpackDegrees` sonucu dereceye çevirir.
- `client/multiplayer/ClientPacketListener.java:767-772`, mevcut entity için
  doğrudan setter yerine `lerpHeadTo(yaw, 3)` çağırır.
- `world/entity/Entity.java:2568-2570` genel dispatch'i, `world/entity/LivingEntity.java:240-254,3312-3315`
  target ve kalan adımları saklayan state'i tanımlar.
- `world/entity/LivingEntity.java:3053-3056,3994-3995`, kalan adım sayısına
  göre `double` `rotLerp` hesabını yapıp sonucu `float` head state'ine daraltır.
- `client/player/RemotePlayer.java:37-50`, bu living head branch'ini remote
  Player `aiStep` yolunda yürütür.
- `util/Mth.java:608-613`, wrapped açı farkının kısa yolu kullandığını tanımlar.

Oracle, `RemotePlayer` tabanlı özgün bir fixture ile `lerpHeadTo` ve gerçek
`lerpHeadRotationStep` Java metodunu çağırır. Renderer, push ve profiler gibi
bu state için gerekli olmayan yan etkiler fixture dışında bırakılır; tick
başındaki previous head kopyası ve remote `aiStep` koşulu korunur.

## Yeni dal ve varyant kararı

Bu dilim, P08.ap server-conformance kaydındaki Java remote Player
`RotateHead`/`aiStep` dalını ilk kez `client-parity` sınıfında çalıştırır.
Farklı gözlem; Player type ID `156` için current, previous, target ve kalan
üç adım state'inin aynı world tick sırasıyla ilerlemesi, 350°→10° wrapped
geçişinin raw float state'inde korunması ve tamamlanmadan gelen yeni head
packet'inin sayacı yeniden kurmasıdır. Aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; P08.e local-player tick kaydı
aynı matematik ailesini ölçse de local `LocalPlayer` tick dalıdır ve remote
Player packet dispatch'ini kapsamaz.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve Player type ID
  `156`'dır. Başlangıç head current/previous/target değeri `350.0`, body yaw
  `15.0` ve interpolation adımı `0`'dır.
- Bir head packet'i current değeri snap etmeden target'ı yazar ve kalan adımı
  `3` yapar. Her world tick'inde previous head önce current değeri alır;
  `1.0 / remaining_steps` ile Java double `rotLerp` uygulanır ve sonuç
  float'a daraltılır.
- 350°→10° geçişinde wrapped fark kullanılır. Java'nın açıyı normalize
  etmeden raw float olarak sakladığı ara/son değerler karşılaştırmada korunur.
- Yeni target önceki interpolation tamamlanmadan gelirse current korunur,
  yeni target yazılır ve sayaç yeniden `3` olur. Head packet'i body yaw'ı
  değiştirmez.
- Local mouse/input bakışı, body-head 50° sınırı, movement/physics,
  renderer/model ve gerçek packet frame karşılaştırması bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/entity-player-head-rotation` kaydı wrapped geçişi, target
replacement'ı, negatif packed açıları ve tamamlanma sayaçlarını taşır. Java
oracle iki tekrar çalıştırıldı; tekrar farkı `null` kaldı.

**Java referans kimliği:**
`e90df0c9f8f1e1dbc3bc3a540a17324201e2c1f7c7d5e79a01db7328673b9548`

Math corpus'u **2990** kayıttır. Yeni kaydın gözlenen state bitleri:

| Adım | `head_rotation` | `previous_head_rotation` | `head_target` | `head_lerp_steps` |
| --- | --- | --- | --- | ---: |
| initial | `43af0000` (350) | `43af0000` | `43af0000` | 0 |
| packet target 10° | `43af0000` | `43af0000` | `41200000` | 3 |
| first tick | `43b25555` | `43af0000` | `41200000` | 2 |
| second tick | `43b5aaaa` | `43b25555` | `41200000` | 1 |
| replacement target -180° | `43b5aaaa` | `43b25555` | `c3340000` | 3 |
| replacement first tick | `43d31c71` | `43b5aaaa` | `c3340000` | 2 |
| replacement complete | `44070000` | `43f08e38` | `c3340000` | 0 |
| target -90° | `44070000` | `43f08e38` | `c2b40000` | 3 |
| negative target complete | `441d8000` | `44160000` | `c2b40000` | 0 |

Body yaw bütün adımlarda `41700000` (`15.0`) kaldı.

## Rust doğrulaması

`ClientWorld::apply_entity_head_rotation` production packet uygulama yolunu
kullanır. `ClientEntity` target ve kalan adım sorgularını sağlar; `tick`
önceki transform snapshot'ını alır ve remote Player için aynı wrapped
interpolation helper'ını yürütür. Java `lerpHeadRotationStep` ile eşleşmesi
için helper double `rotLerp` sonucunu float state'e daraltır.

Debug adapter SHA-256:
`1b1879ccbd59c5d7e0f72ca0273390586230d317ff022abe8de214e7ccef2a04`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=2990
difference=null
status=parity_passed
```

Release adapter SHA-256:
`bc878755290b5d8ae686b2223fd05dfb7645b1aa2a7e8a820872a5e4f50cd38b`

Release adapter aynı capture üzerinde **2990/2990** kayıt ve
`difference=null` ile `parity_passed` oldu.

Son kontroller:

- `cargo test --locked --offline --quiet`: **194 geçti**.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: geçti.
- `cargo fmt --all -- --check`: geçti.
- `git diff --check`: geçti.
- `python3 tools/behavior/test_runner.py`: **5 geçti**.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci sınıflarını
çalıştıran oracle'a ve Rust adapter'a verilir. Vanilla sunucusuna canlı
bağlantı sonucu [P08.ap server-conformance kaydında](client-world-entity-player-head-rotation-conformance.md)
tutulur ve bu Java istemci state karşılaştırmasının yerine geçmez.

## Açık kapsam

Gerçek `ClientPacketListener` frame/thread dispatch'i, full `RemotePlayer.tick`
ve entity lifecycle, motion interpolation, local input/camera, body-head
turn sınırı, passenger/fizik, renderer/model/skin ve oynanabilir istemci
bağlantısı açık kalır.
