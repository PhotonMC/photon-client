# P10.d — LivingEntity fluid falling adjustment — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.getFluidFallingAdjustedMovement`
yolunu local player motion snapshot'ı üzerinde karşılaştırır. Kanıt sınıfı
`client-parity`'dir: gerçek Java metodu kontrollü fixture ile çalıştırılır ve
aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2430-2458`, su ve lav travel dallarının
  fluid hareketinden sonra bu düzeltmeyi çağırdığını gösterir.
- `world/entity/LivingEntity.java:2687-2700`, base gravity, falling state ve
  sprint flag'ine göre y bileşenini seçer; x ve z bileşenlerini değiştirmez.
- `world/entity/LivingEntity.java:2535-2553`, su ve lav akışlarının aynı
  yardımcıyı farklı hareket aşamalarında kullandığını gösterir.

## Davranış sözleşmesi

- Base gravity sıfırsa veya oyuncu sprinting durumundaysa hareket aynı
  bileşen bitleriyle korunur.
- Diğer durumda gravity step `baseGravity / 16.0` olarak hesaplanır ve normal
  sonuç `movement.y - gravityStep` olur.
- Falling durumda hareket y'si `0.005` çevresinden yeterince uzak, ancak
  gravity step'e `0.003` içinde ise y sonucu `-0.003` sabitine çekilir.
- Karşılaştırmalar Java'nın strict `>=` ve `<` dallarıyla yapılır; x/z ve
  signed-zero state'i değiştirilmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/fluid-falling-adjustment` kaydı sekiz gözlem taşır. Sıfır ve
negatif sıfır gravity, sprint bypass'ı, falling özel penceresi, pencere dışı
subtraction, yükselen hareket ve custom gravity normal subtraction yolları
kapsanır.

Java oracle, pinned JAR içindeki gerçek public
`getFluidFallingAdjustedMovement` metodunu constructor çalıştırmadan ayrılmış
`LocalPlayer` fixture'ı üzerinde çağırır. Fixture yalnızca sprint state'ini
kontrollü biçimde sağlar; beklenen hareket Rust çıktısından türetilmez.

Java referans kimliği:
`9a48a2efe29dfcec5f7df3dede2ec48f87c5c808243237aac170b06069dc3707`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2953** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-world/src/input.rs` içindeki `adjust_local_player_fluid_falling_movement`, Java
yardımcısının f64 işlem sırasını ve özel falling penceresini uygular.
`ClientWorld::apply_local_player_fluid_falling_adjustment`, sonucu local
player'ın sorgulanabilir motion state'ine yazar ve sprint flag'ini world
state'inden okur. Behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`5b652f8557fd247fccdd26927e32ff7982e47303d4e34ebbd373d12380ec79db` ile aynı
capture üzerinde **2953/2953** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`6c3124cb141d63c2a422cd22db579c5a96f9b0bcd78caa5f766864279304a2cd` ile aynı
capture üzerinde **2953/2953** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi local player motion state'inin normal fluid
düzeltmesinden sonra güncellendiğini ve sprint flag'i açıkken aynı hareketi
koruduğunu doğrular. `cargo test --locked --offline` koşusunda **176** test
geçti.

## Vanilla server-conformance ayrımı

Bu yardımcı henüz serverbound hareket paketi üretmediği için P10.d için yeni
canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı ilk
movement wire kabulü [P10.a kaydında](client-player-input-conformance.md)
`server-conformance` olarak tutulur; bu kayıt yalnız Java istemci davranışıyla
Rust motion state eşitliğini iddia eder.

## Açık kapsam

Fluid travel'ın input hızları, su/lav buoyancy ve drag sırası, ladder
climbable clamp'i, block friction, collision-aware movement, gerçek local
player tick'i, sprint transition/attribute ve serverbound movement paketleri
sonraki P10 ve P11 dilimlerinde açıktır.
