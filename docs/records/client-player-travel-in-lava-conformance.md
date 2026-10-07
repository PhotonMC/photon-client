# P10.e.2 — LivingEntity travelInLava — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.travelInLava` yolunun çözümlenmiş
lav ortamı girdileriyle hareket state'ine etkisini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: gerçek Java metodu kontrollü bir LocalPlayer fixture'ında
çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2544-2564`, shallow lava kararını, lav
  hareketindeki drag, falling adjustment ve gravity sırasını belirler.
- `world/entity/LivingEntity.java:2566-2571`, yatay çarpışma sonrasında fluid'den
  sıçrama kapısını belirler.
- `world/entity/Entity.java:1747-1762`, relative input dönüşümünü ve yaw
  lookup sırasını belirler.
- `world/entity/Entity.java:3735-3743`, finite olmayan delta yazımının mevcut
  hareket state'ini koruduğunu belirler.
- `world/entity/LivingEntity.java:2687-2700`, shallow lav yolundaki falling
  adjustment hesabını belirler.

## Davranış sözleşmesi

- Relative input, mevcut delta'ya `0.02F` hızla eklenir ve kontrollü fixture'da
  `move` dünya çarpışması üretmez.
- Shallow lavda delta X/Z `0.5` ve Y `0.8F` ile ölçeklenir; ardından sprinting,
  base gravity, falling ve `0.005`/`0.003` özel penceresini kullanan fluid
  falling adjustment çağrılır. Deep lavda tek `0.5` scalar uygulanır ve bu
  falling adjustment çağrılmaz.
- Sıfır olmayan base gravity, mevcut delta Y'sinden `baseGravity / 4.0`
  çıkarır. Sıfır gravity bu yazımı atlar; sprinting yalnızca falling adjustment
  içindeki gravity adımını bypass eder.
- Yatay çarpışma ve çözülen boşluk sorgusu birlikte true ise son setter Y'yi
  f32 `0.3F` yapar. Java finite delta setter'ı başarısız bir yazımı mevcut
  delta state'inde tutar.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/travel-in-lava` kaydı altı gözlem taşır: shallow/deep lav,
zero gravity, sprint bypass, başarısız fluid jump ve başarılı fluid jump.

Java oracle, pinned JAR içindeki gerçek private `travelInLava` metodunu
reflection ile çağırır. Fixture; shallow lava sonucu, sprint/collision/free
sonuçlarını, yaw'ı ve delta state'ini sağlar. Gerçek world fluid lookup ve
collision query bu kaydın dışındadır; bunlar fixture'a çözümlenmiş girdiler
olarak verilir.

Java referans kimliği:
`8b164dde9fc8f71e9c9ee5a7149cf662a70ac4b4d8e7d9864ee638dbfb8545bd`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2959** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_travel_in_lava`, Java'nın shallow/deep
drag ayrımını, relative input dönüşümünü, falling adjustment, gravity ve fluid
jump sırasını uygular. Setter adımları finite olmayan ara değerlerde mevcut
delta state'ini korur. `ClientWorld` local player motion state'inden delta'yı
alıp sonucu tekrar sorgulanabilir motion state'ine yazar; behavior adapter aynı
production helper'ını çağırır.

Debug adapter SHA-256 değeri
`4c761d15ba4ea563106eaa4409729229f8085f55a93fc020237233af14e9907d` ile aynı
capture üzerinde **2959/2959** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`6f0f7c7215d258c597b23632a483df31562ef231a13f7cd3bbdca346b0ee689e` ile aynı
capture üzerinde **2959/2959** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi shallow lava drag/gravity state'ini ve yatay
çarpışma sonrası fluid jump state'ini doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için `travelInLava` için ayrı canlı
sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login,
world delivery ve movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java
ile eşitliğini ölçer.

## Açık kapsam

Gerçek block/fluid lookup, shallow fluid height/threshold, `travel` dispatch'i,
collision-aware `move`, LocalPlayer tick/input bağlantısı ve serverbound
movement paketleri sonraki P10/P11 dilimlerinde açıktır. Su, flight/elytra,
auto-jump ve araç kontrolü de bu kaydı kapatmaz.
