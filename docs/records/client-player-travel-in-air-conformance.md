# P10.e.4 — LivingEntity travelInAir — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.travelInAir` yolunun çözümlenmiş
block, effect ve level girdileriyle local player hareket state'ine etkisini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: gerçek Java metodu kontrollü
bir LocalPlayer fixture'ında çalıştırılır ve aynı girdiler production Rust
helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2424-2500`, `travel` içindeki hava dispatch'ini,
  zemin friction çözümünü, levitation/gravity dallarını ve yatay/dikey air drag
  sırasını belirler.
- `world/entity/LivingEntity.java:2675-2685,2719-2729`, relative hareketin,
  friction etkili hızın ve Player flying speed fallback'inin nasıl üretildiğini
  belirler.
- `world/entity/Entity.java:1747-1762`, relative input dönüşümünü ve yaw
  lookup sırasını belirler.
- `world/entity/Entity.java:3735-3743`, finite olmayan delta yazımının mevcut
  hareket state'ini koruduğunu belirler.
- `world/entity/player/Player.java:1974-1980`, local Player'ın uçuş yeteneği,
  sprint ve passenger durumuna göre hava hızını nasıl seçtiğini belirler.

## Davranış sözleşmesi

- Zemin durumunda block friction, `FRICTION_MODIFIER` ile
  `computeModifiedFriction` üzerinden f32 olarak değiştirilir; havada bu değer
  `1.0F` olur. Sonuç `handleRelativeFrictionAndCalculateMovement` yoluna verilir.
- Relative hareket mevcut delta'ya eklenir. Kontrollü fixture'da `move` dünya
  çarpışması üretmez; climbable clamp ve step kararları bu alt çağrının
  sözleşmesine göre uygulanır.
- Levitation varsa Y, effect amplifier hedef hızına `0.2` katsayısıyla yaklaşır.
  Aksi durumda yüklü/server level dalı effective gravity çıkarır. Client tarafında
  ilgili chunk yüklü değilse, entity minimum Y'nin üstündeyken Y `-0.1`, minimum
  Y'de ise `0.0` yapılır.
- Friction atılmadığında `AIR_DRAG_MODIFIER`, yatay `0.91F` ve dikey `0.98F`
  drag değerlerini `computeModifiedFriction` ile üretir. Omnidirectional air
  mover dalı yatay drag'ı dikeyde de kullanır. `shouldDiscardFriction` true ise
  drag uygulanmadan hareket yazılır.
- Java finite delta setter'ı sonlu olmayan sonucu reddeder ve önceki delta
  state'ini korur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/travel-in-air` kaydı sekiz gözlem taşır: server gravity ve
drag, zemin ice friction'ı, levitation, friction discard, omnidirectional air
mover, air-drag attribute ve client tarafı yüklenmemiş chunk'ın minimum Y
fallback dalları.

Java oracle, pinned JAR içindeki gerçek private `travelInAir` metodunu reflection
ile çağırır. Fixture; block friction, Player ability/hız, climbable ve collision
durumlarını, levitation amplifier'ını, gravity/drag attribute'larını, client
level chunk durumunu, minimum Y'yi, yaw'ı ve delta state'ini sağlar. Block/level,
effect ve collision çözümlemesinin gerçek client bağlantısı bu kaydın dışındadır;
çözümlenmiş girdiler fixture'a verilir.

Java referans kimliği:
`299765a8e5ec094fd7c964e286fcf08f9c4461c6616b9fac06a519e209ffb518`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2961** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_travel_in_air`, Java'nın relative hareket,
gravity/levitation, client fallback ve friction/air-drag sırasını uygular.
`ClientWorld` local player motion state'inden delta'yı alıp sonucu tekrar
sorgulanabilir motion state'ine yazar; behavior adapter aynı production
helper'ını çağırır.

Debug adapter SHA-256 değeri
`37c1029e0e5a183255f0f1f738a448c441ca0c015c616e59f6e5b1965c559e6f` ile aynı
capture üzerinde **2961/2961** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`e85c1f61f26265151562b174630d60245f36c08793f8176e582fb283ce6fc090` ile aynı
capture üzerinde **2961/2961** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi server gravity/drag, zemin friction, levitation
ve client minimum-Y fallback state'lerini doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için `travelInAir` için ayrı canlı
sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login, world
delivery ve movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java ile
eşitliğini ölçer.

## Açık kapsam

Gerçek block/attribute/effect/level lookup, collision-aware `move`, tam `travel`
dispatch'i, LocalPlayer tick/input bağlantısı, uçuş/elytra/auto-jump, fizik
entegrasyonu ve serverbound movement paketleri sonraki P10/P11 dilimlerinde
açıktır. Bu kayıt çözümlenmiş hava hareketi state'ini kapsar.
