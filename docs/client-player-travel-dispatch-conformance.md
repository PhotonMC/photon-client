# P10.e.6 — LivingEntity travel dispatch — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.travel` seçim sırasını ve seçilen
no-collision hareket yolunun local player motion state'ine etkisini karşılaştırır.
Kanıt sınıfı `client-parity`'dir: gerçek Java `travel` çağrısı kontrollü bir
LocalPlayer fixture'ında çalıştırılır ve aynı çözümlenmiş girdiler production
Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2429-2445`, `travel` ve
  `shouldTravelInFluid` seçim sırasını belirler.
- `world/entity/LivingEntity.java:2503-2513`, fluid dalında su/lav dispatch'ini
  ve su sonrası ek ridden float çağrısını belirler.
- `world/entity/LivingEntity.java:2580-2593`, fall-flying dalındaki climbable
  hava fallback'ini ve normal glide update yolunu belirler.
- `world/entity/LivingEntity.java:2467-2495`, hava fallback'inde kullanılan
  çözümlenmiş `travelInAir` hareketini belirler.

## Davranış sözleşmesi

- `shouldTravelInFluid`, su veya lav durumunu, fluid'lerden etkilenme kapısını
  ve fluid üzerinde durabilme kapısını birlikte değerlendirir. Sonuç doğruysa
  `travelInFluid` seçilir; bu yardımcı su durumunu lavdan önce sınar.
- Fluid dalı seçilmezse `isFallFlying` kontrol edilir. Fall-flying dalı
  climbable olduğunda `travelInAir` fallback'ini çalıştırır ve fall-flying'i
  durdurur. Diğer durumda `updateFallFlyingMovement` ile hesaplanan hareketi
  kullanır.
- Her dal için block/fluid/attribute/effect/level sorguları capture sınırında
  çözümlenmiş girdiler olarak sağlanır. Hareket sonucu collision-aware `move`
  çözümlemesi olmadan karşılaştırılır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/travel-dispatch` kaydı dokuz gözlem taşır: hava varsayılanı,
su, lav, suyun lav üzerindeki önceliği, fluid etkilenmeme, fluid üzerinde
durabilme, normal fall-flying, climbable fall-flying fallback'i ve fluid'in
fall-flying üzerindeki önceliği.

Java oracle, pinned JAR içindeki gerçek `LivingEntity.travel` yolunu çağırır.
Fixture, `Player.travel` üst katmanının bu dispatch'i değiştirmemesi için
swimming, passenger ve ability-flying durumlarını kapatır; LivingEntity'nin
fluid, climbable, gravity, friction, look ve motion sorgularını sağlar.

Java referans kimliği:
`7f5af7c72bd653a04d8ed790da165c741558f14979b5bee83e3de39defb6f4c7`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2963** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_travel_dispatch`, Java'nın fluid,
fall-flying ve hava branch sırasını uygular; seçilen branch'in mevcut
no-collision hareket helper'ını çağırır. `ClientWorld` local player motion
state'inden delta'yı alıp sonucu tekrar sorgulanabilir motion state'ine yazar.
Behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`8a98c3738c453240a503892746273bb0efe580e3b40d5f31394217bdae79ecb8` ile aynı
capture üzerinde **2963/2963** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`4aa988f624a43d7ff3e21ca145e8d1e6f0d6ab828c72096c9f098e1ea5cbfdf3` ile aynı
capture üzerinde **2963/2963** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi su, normal fall-flying ve climbable fallback
branch'lerinin seçimini ve sorgulanabilir motion state güncellemesini doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için travel dispatch için ayrı canlı
sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login,
world delivery ve kontrollü movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci travel state'inin Java
ile eşitliğini ölçer.

## Açık kapsam

`floatInWaterWhileRidden` alt çağrısı, collision-aware `move`, server-side
fall-flying collision hasarı, gerçek fluid/block/attribute/effect/level lookup,
elytra ve pose state'i, LocalPlayer tick/input bağlantısı, fizik geçişleri ve
serverbound movement paketleri sonraki P09–P11 dilimlerinde açıktır.
