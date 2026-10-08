# P10.d.1 — LivingEntity climbable movement clamp — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.handleOnClimbable` yolunu local
player motion snapshot'ı üzerinde karşılaştırır. Kanıt sınıfı
`client-parity`'dir: gerçek Java private metodu kontrollü fixture ile
çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2675-2681`, air travel hareketinden sonra
  climbable hareket yardımcısının çağrı sırasını gösterir.
- `world/entity/LivingEntity.java:2702-2713`, climbable state, ±`0.15F`
  sınırları, scaffolding istisnası ve Player ladder slide kapısını taşır.
- `util/Mth.java:102-108`, double clamp'in strict lower-bound ve `Math.min`
  sırasını belirler.

## Davranış sözleşmesi

- Oyuncu climbable değilse hareket bileşenleri olduğu gibi korunur.
- Climbable durumda x/z hareketi Java kaynak değerinden genişletilmiş
  `[-0.15F, 0.15F]` aralığına, y hareketi ise en az `-0.15F` değerine
  sınırlandırılır.
- Player climbable state'te, scaffolding içinde değilse, negatif y hareketi
  varsa ve ladder slide suppression açıksa y hareketi `+0.0` olur.
- Scaffolding bu y sıfırlama kapısını bypass eder; yukarı hareket suppression'dan
  etkilenmez. Clamp edilmemiş signed-zero bileşenleri korunur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/climbable-movement` kaydı altı gözlem taşır. Climbable
olmayan yol, yatay/aşağı clamp, ladder slide suppression, scaffolding,
upward motion ve signed-zero dalları kapsanır.

Java oracle, pinned JAR içindeki gerçek private `handleOnClimbable` metodunu
constructor çalıştırmadan ayrılmış `LocalPlayer` fixture'ı üzerinde reflection
ile çağırır. Fixture `onClimbable`, in-block scaffolding state'i ve slide
suppression sonucunu sağlar; beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`c24bc8e3f760176b65045ff4cfb4b86c3f86be399eeda4f264922a7651354434`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2954** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-world/src/input.rs` içindeki `clamp_local_player_climbable_movement`, Java'nın
f32 clamp sınırlarını f64'e genişletme ve slide suppression sırasını uygular.
`ClientWorld::apply_local_player_climbable_state`, local player'ın çevre
snapshot'ını saklar; `apply_local_player_climbable_movement` sonucu
motion state'ine yazar. Behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`927e6370664eea59d09a03bb1f70ffffbd7a0b6a8fd51ae525550270a2a951c4` ile aynı
capture üzerinde **2954/2954** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`dd161aca5a7e9faf0d3f1479ae898e4fdcee573678628c11db13636edffc5110` ile aynı
capture üzerinde **2954/2954** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi climbable motion clamp'ini, ladder slide
suppression'ı ve scaffolding istisnasını sorgulanabilir local player state'inde
doğrular. `cargo test --locked --offline` koşusunda **177** test geçti.

## Vanilla server-conformance ayrımı

Bu yardımcı henüz serverbound hareket paketi üretmediği için P10.d.1 için yeni
canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı
ölçülebilir login/world tick smoke testi ve ilk movement wire kabulü
[P10.a kaydında](client-player-input-conformance.md) `server-conformance`
olarak tutulur; bu kayıt yalnız Java istemci davranışıyla Rust motion state
eşitliğini iddia eder.

## Açık kapsam

Climbable state'in block/world sorgusundan otomatik üretilmesi, tam ladder
travel, water/lava travel, block friction, collision-aware movement, gerçek
local player tick'i, sprint transition/attribute ve serverbound movement
paketleri sonraki P10 ve P11 dilimlerinde açıktır.
