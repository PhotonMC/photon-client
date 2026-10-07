# P10.e.3 — LivingEntity handleRelativeFrictionAndCalculateMovement — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.handleRelativeFrictionAndCalculateMovement`
yolunun çözümlenmiş hava/merdiven girdileriyle hareket state'ine etkisini
karşılaştırır. Kanıt sınıfı `client-parity`'dir: gerçek Java metodu kontrollü
bir LocalPlayer fixture'ında çalıştırılır ve aynı girdiler production Rust
helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2675-2685`, friction etkili hızı,
  relative hareketi, climbable clamp'ını ve son fluid step dalını sıraya koyar.
- `world/entity/LivingEntity.java:2687-2700`, climbable hareketinin f32 sınır
  değerlerini f64 delta üzerinde nasıl uyguladığını belirler.
- `world/entity/LivingEntity.java:2719-2729`, zemin friction'ı ile Player'ın
  hava uçuş hızından relative hareket hızını seçer.
- `world/entity/Entity.java:1747-1762,3735-3743`, relative input dönüşümünü ve
  finite delta setter davranışını belirler.

## Davranış sözleşmesi

- Java önce zemin durumuna ve block friction'a göre `getFrictionInfluencedSpeed`
  sonucunu seçer; hava dalında Player'ın flying ability, sprint ve passenger
  fallback'i kullanılır. Bu hızla relative input mevcut delta'ya eklenir.
- Ardından `handleOnClimbable` çağrılır. Climbable durumda yatay bileşenler
  f32 `0.15F` sınırlarının f64 karşılığına clamp edilir; aşağı yönlü Y aynı alt
  sınırı kullanır ve Player ladder suppression koşulu sağlanırsa sıfırlanır.
- Kontrollü fixture'da `move` dünya çarpışması üretmez. Yatay çarpışma veya
  jumping ile climbable birlikte true olduğunda sonuç Y'si double `0.2` olur.
  Powder snow koşulu bu kaydın fixture girdilerinde etkin değildir.
- Her delta setter'ı finite olmayan sonucu reddeder ve önceki hareket state'ini
  korur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/handle-relative-friction` kaydı altı gözlem taşır: hava ve
zemin hız seçimi, climbable clamp, yatay çarpışma step'i, jumping step'i ve
climbable olmayan jumping yolu.

Java oracle, pinned JAR içindeki gerçek private
`handleRelativeFrictionAndCalculateMovement` metodunu reflection ile çağırır.
Fixture; zemin/friction/hız ve Player ability girdilerini, climbable block
durumunu, çarpışma/jump bayraklarını, yaw'ı ve delta state'ini sağlar. Gerçek
block lookup, powder snow lookup ve collision query bu kaydın dışındadır;
çözümlenmiş girdiler fixture'a verilir.

Java referans kimliği:
`61f0da99b45f285a7d3d477b54ad624201c68e2abcc03e92f6bfff482bb851cc`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2960** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_handle_relative_friction`, mevcut
friction-influenced speed ve climbable clamp helper'larını Java çağrı sırasıyla
birleştirir. `ClientWorld` local player motion state'inden delta'yı alıp sonucu
tekrar sorgulanabilir motion state'ine yazar; behavior adapter aynı production
helper'ını çağırır.

Debug adapter SHA-256 değeri
`1dc32d33eb14240abefb56b02ab7f611896b1c5d7a27f2241ba90555ddb9dac3` ile aynı
capture üzerinde **2960/2960** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`24d3b86b2150b6eb70560df1e73f128b65c3b622374a9b2e8e66bb670789fb3d` ile aynı
capture üzerinde **2960/2960** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi normal relative air hareketini ve climbable
çarpışma sonrası step state'ini doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için ayrı canlı sunucu koşusu yapılmaz.
Resmi Mojang vanilla 26.2 sunucusuna karşı login, world delivery ve movement
wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java
ile eşitliğini ölçer.

## Açık kapsam

Gerçek block friction/attribute lookup, powder snow walkability, collision-aware
`move`, tam `travelInAir` gravity/levitation/air-drag yolu, LocalPlayer tick/input
bağlantısı ve serverbound movement paketleri sonraki P10/P11 dilimlerinde
açıktır. Bu kayıt fluid travel, fall-flying ve araç kontrolünü kapsamaz.
