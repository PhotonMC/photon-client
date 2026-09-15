# P08.av.1 — ClientWorld entity passenger relationships — [client-parity]

Bu kayıt, `ClientboundSetPassengersPacket` uygulamasındaki araç/passenger
ilişkisinin Java istemcisi ile Rust `ClientWorld` kopyasında eşleşmesini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik input Java
oracle'ına ve production Rust adapter'a verilir.

[P08.av server-conformance kaydı](client-world-entity-passengers-conformance.md)
vanilla sunucusunun packet akışını ve canlı world kabulünü tutar. O kayıt wire
uyum kanıtıdır; bu kayıt Java istemcisindeki ilişki state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu dallar izlendi:

- `client/multiplayer/ClientPacketListener.java:1146-1175`, mevcut araç
  bulunamazsa paketi atlar; araç bulunursa önce `ejectPassengers` çağırır ve
  packet sırasındaki mevcut entity'leri `startRiding(vehicle, true, false)` ile
  dener.
- `world/entity/Entity.java:2420-2465`, `startRiding` aynı aracı tekrar,
  self/cycle ilişkisini ve kabul koşullarını kontrol eder; force dalı mevcut
  araçtan ayrılma ve yeni araca ekleme state'ini yürütür.
- `world/entity/Entity.java:2472-2525`, passenger çıkarma ve araç listesinin
  replacement sırası ile `boardingCooldown` yan etkisini belirler.
- `client/multiplayer/ClientLevel.java:450-490` ilişki listesinin daha sonra
  passenger tick/transform ağacında kullanıldığını gösterir; tick davranışı
  P08.aw kapsamındadır.

Oracle, handler'ın ilişki bölümünü gerçek Java `Entity.ejectPassengers`,
`Entity.startRiding` ve `getVehicle`/`getPassengers` çağrılarıyla çalıştırır.
Packet wrapper'ı, log mesajı ve gerçek Minecraft oyuncusuna bağlı HUD/narrator
yan etkileri bu state karşılaştırmasına dahil edilmez.

## Yeni dal ve varyant kararı

Bu kayıt, P08.av server-conformance ilişki dalını ilk kez `client-parity`
sınıfında çalıştırır. P08.au.1 teleport ve önceki Player parity kayıtları
transform/motion state'ini ölçer; buradaki farklı gözlem SetPassengers'ın
mevcut araç listesini atomik replacement gibi temizlemesi, passenger'ı eski
araçtan ayırması, packet sırasını koruması ve geçersiz cycle/duplicate/unknown
girdileri ilişki state'ine eklememesidir. Aynı sonucu veren mevcut
client-parity kayıtlarının sayısı **0**'dır.

Tek parametrik corpus kaydı initial nested ilişkileri, reparenting,
duplicate/unknown filtrelerini, cycle reddini, boş replacement'ı ve nested
replacement sırasını kapsar. Sabit değiştirerek aynı Java dalını çoğaltan ayrı
bir entity kaydı açılmadı.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir.
- Başlangıç graph'ı `39 <- 38`, `37 <- 40 <- 41` ilişkilerini taşır; entity
  sırası `[37,38,39,40,41,42]` çıktıda korunur.
- Geçerli araç packet'inde önce doğrudan passenger listesi çıkarılır. Listedeki
  mevcut entity'ler sırayla force riding ile bağlanır; passenger'ın eski aracı
  varsa eski araçtan ayrılır.
- Bilinmeyen passenger, aynı araca tekrar ekleme ve araç zincirinde cycle
  oluşturacak giriş Java `startRiding` sonucu gibi atlanır. Bilinmeyen araç
  packet'i state'i değiştirmez ve `accepted=false` gözlemi üretir.
- Boş passenger listesi doğrudan araçtaki ilişkileri temizler; çıkarılan
  passenger'ın kendi passenger listesi korunur.
- Çıktı her entity için `vehicle_id` ve sıralı `passenger_ids` state'ini taşır.
  Local Player mount mesajı, boat yaw/head güncellemesi, HUD/narrator,
  boarding cooldown ve passenger tick/positionRider yan etkileri bu dilimin
  dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki tek `client-world/entity-passengers` kaydı
replacement/reparent, cycle, unknown, duplicate, empty ve nested ilişkileri
aynı graph üzerinde yürütür. Java oracle iki çalıştırmada kararlı kaldı;
tekrar farkı `null`'dır.

Math capture kimliği:
`02c0981c84f5bcd5292d274e48006abb7e54b27bfa7f6f3d0e4fba10af32622e`

Bu capture math ailesinde **3005** case içerir. Passenger state'inin seçili
geçişleri:

| Adım | araç/passenger ilişkileri |
| --- | --- |
| initial | `37:[40]`, `39:[38]`, `40:[41]` |
| replacement | `37:[38,39,42]`; `38,39,42 → 37`; `40:[41]` |
| cycle | önceki state korunur |
| empty | `37` listesi boşalır; `40:[41]` kalır |
| unknown vehicle | state değişmez, `accepted=false` |
| nested order | `40:[37,42]`; `37:[38,39]` iç ilişkisi korunur |
| cycle keeps links | `37` yalnız `38`i alır; `40:[37,42]` korunur |
| nested replacement | `40:[41]`; `37:[38]` korunur |

Tam corpus Java capture kimliği:
`b8837b77552d99504c4a69d06c6343c15e503e4f50db59930e0581ecf8738f44`

Toplam **3866** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter production `ClientWorld::apply_entity`,
`ClientWorld::apply_entity_passengers`, `ClientEntity::vehicle_id` ve
`ClientEntity::passenger_ids` API'lerini kullanır. İlişki input'u Rust
world'üne gerçek entity kayıtları olarak eklenir; state snapshot'ı aynı input
sırasını korur.

Debug adapter SHA-256:
`c5547daa6bba1ca8a2c0036b9fff92b99f85f6ece27a6f635b37b4495d059274`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3005
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3866
difference=null
status=parity_passed
```

Release adapter SHA-256:
`60be8fe1366efcffc278865738e7fc6226bbdf32b89b808c7b968c46a540d764`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3866
difference=null
status=parity_passed
```

İlgili Rust testleri ve clippy kontrolleri bu kaydın commit'inden önce
çalıştırılacaktır. Tam vanilla server koşusu [P08.av server-conformance
kaydında](client-world-entity-passengers-conformance.md) tutulur.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun SetPassengers packet'ini
kabul etmesi [P08.av server-conformance kaydında](client-world-entity-passengers-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Passenger tick/rideTick ve `positionRider`, nested controlling passenger
çözümü, local Player mount HUD/yaw/head yan etkileri, vehicle physics, collision,
boarding cooldown gözlemi, entity lifecycle'ın chunk tarafı, teleport/movement
ile ilişki sırası ve oynanabilir istemci bağlantısı açık kalır. Bu kayıt P08'in
`client-parity` kapanışı değildir.
