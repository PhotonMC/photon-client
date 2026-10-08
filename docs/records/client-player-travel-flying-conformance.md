# P10.d.4 — LivingEntity travelFlying — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.travelFlying` yolunun ortam dalını,
relative input dönüşümünü ve hareket sonrası drag sırasını karşılaştırır. Kanıt
sınıfı `client-parity`'dir: gerçek Java metodu çarpışmasız bir fixture üzerinde
çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2447-2464`, su, lav ve hava dallarının
  önceliğini; `moveRelative`, `move` ve drag sırasını belirler.
- `world/entity/Entity.java:1747-1762`, relative input'ın yatay rotasyonunu,
  `1.0E-7` uzunluk eşiğini ve birden uzun input'ın normalize edilmesini
  belirler.
- `world/phys/Vec3.java:83-85,180-185`, normalize ve squared-length işlemlerinin
  double sırasını belirler.
- `world/entity/Entity.java:3735-3743`, sonlu olmayan delta'nın entity state'ine
  yazılmaması davranışını belirler.

## Davranış sözleşmesi

- `isInWater` true ise su hızı ve `0.8F` drag seçilir. Su false ve `isInLava`
  true ise lav hızı ve double `0.5` drag seçilir. Diğer durumda hava hızı ve
  `0.91F` drag seçilir; su dalı lav dalından önce gelir.
- Seçilen f32 hızla relative input eklenir, sonra `MoverType.SELF` hareketi
  çalışır ve mevcut delta seçilen drag ile ölçeklenir. Capture fixture'ında
  `move` çarpışmasız olduğundan çözülen delta, relative eklemenin sonucudur.
- Input squared length `1.0E-7` değerinin altındaysa relative delta sıfırdır.
  Daha uzun input `Vec3.normalize` yoluna girer; diğer input ham bileşenleriyle
  hızlandırılır.
- Yaw, Java'nın f32 derece-radyan çarpımı ve Mth sin/cos lookup değerleriyle
  x/z düzlemine uygulanır; y bileşeni aynı kalır. Relative toplam sonlu değilse
  Java setter'ı eski delta'yı korur, son drag yine o delta'ya uygulanır.
- Su ve hava drag sabitleri f32 olarak widening sonrası, lav sabiti double olarak
  uygulanır; bu fark ham double sabitlerle değiştirilemez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/travel-flying` kaydı sekiz gözlem taşır. Su, lav, su önceliği
ve hava dalları; normalize edilen input, küçük uzunluk eşiği, signed zero ve
sonlu olmayan relative delta yolları aynı tabloda ölçülür.

Java oracle, pinned JAR içindeki gerçek protected `travelFlying` metodunu
reflection ile çağırır. Fixture ortam bayraklarını, yaw'ı ve delta state'ini
sağlar; `move` çarpışma sonucu üretmeden döner ve setter'ın finite kontrolünü
korur. Beklenen çıktı Rust'tan türetilmez.

Java referans kimliği:
`3a9e3ccd4e26ffa0aaaf986fc4ee53e3b3354675d255b42fb2e14f18afab5548`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2957** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-world/src/input.rs` içindeki `local_player_travel_flying`, relative input dönüşümünü,
environment branch'lerini ve drag sırasını uygular. `ClientWorld` local player
motion state'inden delta'yı alıp sonucu tekrar sorgulanabilir motion state'ine
yazar; behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`b66be90a3d0e9b8c5bda04992b23b548eaa8f76c6b165e87e2efbfd8435981a6` ile aynı
capture üzerinde **2957/2957** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`6ef8c805216e687acdfcd26704e52f06d2e023cd57667cb7a6de9e943f77d5a1` ile aynı
capture üzerinde **2957/2957** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi hava ve su state binding'ini, yaw dönüşünü ve
drag bitlerini doğrular. Tam Rust test ve statik kontrol sonucu bu kaydın
commit doğrulamasında tutulur.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için `travelFlying` için ayrı canlı
sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login, world
delivery ve mevcut movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java ile
eşitliğini ölçer.

## Açık kapsam

Gerçek block/fluid lookup, collision-aware `move`, tam `travel` dispatch'i,
LocalPlayer tick/input bağlama, su/lav fizik dallarının tamamı ve serverbound
movement paketleri sonraki P10 ve P11 dilimlerinde açıktır.
