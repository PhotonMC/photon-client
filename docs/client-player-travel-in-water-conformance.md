# P10.e.1 — LivingEntity travelInWater — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.travelInWater` yolunun çözümlenmiş
su ortamı girdileriyle hareket state'ine etkisini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: gerçek Java metodu kontrollü bir LocalPlayer fixture'ında
çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2515-2542`, su travel hızını, drag sırasını,
  water movement efficiency ve Dolphins Grace dallarını belirler.
- `world/entity/LivingEntity.java:2566-2571`, yatay çarpışma sonrasında fluid'den
  sıçrama kapısını belirler.
- `world/entity/Entity.java:1747-1762`, travel input'ının relative dönüşümünü
  ve yaw lookup sırasını belirler.
- `world/entity/LivingEntity.java:2687-2700`, su hareketinden sonra uygulanan
  falling adjustment davranışını belirler.

## Davranış sözleşmesi

- Sprinting durumda başlangıç slowdown değeri `0.9F`, diğer durumda
  `getWaterSlowDown()` sonucudur. Water movement efficiency f32'ye çevrilir;
  oyuncu zeminde değilse yarıya iner. Pozitif değer, slowdown'u `0.54600006F`
  yönünde ve hareket hızını oyuncu speed'i yönünde f32 işlemleriyle karıştırır.
- Dolphins Grace varsa hesaplanan slowdown `0.96F` ile değiştirilir.
- Relative input mevcut delta'ya eklenir. Kontrollü fixture'da `move` dünya
  çarpışması üretmez; yatay çarpışma ve climbable birlikteyse drag öncesi Y
  değeri double `0.2` olur. Ardından X/Z slowdown, Y `0.8F` ile ölçeklenir ve
  önceki fluid falling adjustment uygulanır.
- Yatay çarpışma sonrasında çözülen boşluk sorgusu true ise son setter Y'yi
  f32 `0.3F` yapar. Java'nın finite delta setter'ı başarısız son yazmayı
  mevcut delta state'inde tutar.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/travel-in-water` kaydı sekiz gözlem taşır: normal su drag ve
gravity, sprint bypass, zeminde ve havada efficiency, Dolphins Grace,
climbable step, fluid jump ve negatif efficiency'nin reddi.

Java oracle, pinned JAR içindeki gerçek protected `travelInWater` metodunu
reflection ile çağırır. Fixture; speed, slowdown, efficiency, sprint/ground
state'lerini, effect ve collision sonuçlarını, yaw'ı ve delta state'ini sağlar.
Gerçek world fluid lookup ve collision query bu kaydın dışındadır; bunlar
fixture'a çözümlenmiş girdiler olarak verilir.

Java referans kimliği:
`d50be9ae04a77970e705ec085ff46f5842bd330adb1c5eb79997843096d0302c`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2958** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`src/input.rs` içindeki `local_player_travel_in_water`, Java'nın f32
slowdown/efficiency işlemlerini, relative input dönüşümünü, drag ve falling
adjustment sırasını uygular. `ClientWorld` local player motion state'inden
delta'yı alıp sonucu tekrar sorgulanabilir motion state'ine yazar; behavior
adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`5b5fdf43ec1352547729a3441cfe0c24fd1dff3061274aad436b62329615c0f7` ile aynı
capture üzerinde **2958/2958** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi normal water drag/gravity state'ini ve yatay
çarpışma sonrası fluid jump state'ini doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için `travelInWater` için ayrı canlı
sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login,
world delivery ve movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java
ile eşitliğini ölçer.

## Açık kapsam

Gerçek block/fluid lookup, `travel` dispatch'i, collision-aware `move`, tam
fluid height/jump threshold ve LocalPlayer tick/input bağlantısı sonraki P10
ve P09 dilimlerinde açıktır. Lav travel, flight/elytra, auto-jump ve
serverbound movement paketleri de bu kaydı kapatmaz.
