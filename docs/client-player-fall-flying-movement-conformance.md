# P10.e.5 — LivingEntity updateFallFlyingMovement — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.updateFallFlyingMovement` hesabının
elytra yönelimi, lift, yatay hız dönüşümü ve drag sırasını karşılaştırır. Kanıt
sınıfı `client-parity`'dir: gerçek Java private metodu kontrollü bir LocalPlayer
fixture'ında çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2580-2593`, fall-flying dalının climbable ve
  normal hareket ayrımını, son hareketin kaydedilmesini ve collision callback'ini
  belirler.
- `world/entity/LivingEntity.java:2601-2626`, look angle, lean angle, gravity,
  lift, düşüş hızının look yönüne dönüşümü, nose-up lift ve yatay hizalama sırasını
  belirler.
- `world/entity/Entity.java:2576-2578`, fall-flying hesabına verilen look angle
  state'ini belirler.
- `util/Mth.java:50-55,642-644`, `Mth.sin` lookup'ını ve square hesabını belirler.
- `world/phys/Vec3.java:188-190`, yatay hareket uzunluğunun double square-root
  sırasını belirler.

## Davranış sözleşmesi

- Lean angle, Player X rotation ile Java'nın f32 derece-radyan sabitinden
  üretilir. Look angle'ın yatay uzunluğu ve mevcut movement'ın yatay uzunluğu
  double olarak hesaplanır.
- Önce Y'ye gravity ile `(-1.0 + square(cos(leanAngle)) * 0.75)` lift terimi
  eklenir. Hareket aşağı yönlüyse ve look'ın yatay uzunluğu varsa düşüşün bir
  bölümü look yönüne ve Y'ye çevrilir.
- Lean angle negatifse ve look yataysa, yatay hızdan türetilen nose-up dönüşümü
  look yönünün tersine eklenir. Look yatay ise sonrasında yatay hareket, mevcut
  yatay hızın look yönündeki bileşenine `0.1` ile yaklaştırılır.
- Son movement, X/Z'de f32 `0.99F`, Y'de f32 `0.98F` ile ölçeklenir. Look'ın
  yatay uzunluğu sıfır olduğunda iki look yönlü dönüşüm de atlanır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/update-fall-flying-movement` kaydı altı gözlem taşır: nötr
glide, nose-down düşüş dönüşümü, nose-up lift, dikey look, sıfır gravity ve
negatif yatay look yönü.

Java oracle, pinned JAR içindeki gerçek private `updateFallFlyingMovement`
metodunu reflection ile çağırır. Fixture look angle, X rotation, effective
gravity ve başlangıç delta'sını sağlar; beklenen çıktı Rust'tan türetilmez.

Java referans kimliği:
`0d533f2354b579c19011df0aabf9f0f710cad56c714b8e13a2c9163b6e9888f9`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2962** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_update_fall_flying_movement`, Java'nın
lift, düşüş/nose-up dönüşümleri, look hizalaması ve drag sırasını uygular.
`ClientWorld` local player motion state'inden delta'yı alıp sonucu tekrar
sorgulanabilir motion state'ine yazar; behavior adapter aynı production
helper'ını çağırır.

Debug adapter SHA-256 değeri
`9b7fefd4901cab292539aab5771d596e1e367108a227d2cd353938732a907558` ile aynı
capture üzerinde **2962/2962** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`c4b1e3b18f9f3e08ef20784bf22c571bc9278b33a94e843a5a04d697f0053ba7` ile aynı
capture üzerinde **2962/2962** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi nötr, nose-down, nose-up ve dikey look dallarını
doğrular.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmediği için fall-flying movement için ayrı
canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı login,
world delivery ve movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt istemci motion state'inin Java ile
eşitliğini ölçer.

## Açık kapsam

`travelFallFlying` içindeki climbable durumunda `travelInAir` fallback'i ve
`stopFallFlying`, collision-aware `move`, server-side fall-flying collision
hasarı, gerçek look/rotation/elytra state lookup'ı, LocalPlayer tick/input
bağlantısı ve serverbound movement paketleri sonraki P10/P11 dilimlerinde
açıktır.
