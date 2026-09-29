# P10.d.2 — LivingEntity friction-influenced speed — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.getFrictionInfluencedSpeed` yolunu
local player'ın zemin ve hava hareketi hız seçimi üzerinde karşılaştırır. Kanıt
sınıfı `client-parity`'dir: gerçek Java private metodu, `LocalPlayer` fixture'ı
üzerinde çalıştırılır ve aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:2675-2678`, air travel içinde sürtünme hızının
  hareket vektörüne uygulanmadan önce seçildiğini gösterir.
- `world/entity/LivingEntity.java:2719-2729`, zemin sürtünmesi eşiğini ve hava
  dalında `getFlyingSpeed` çağrısını taşır.
- `world/entity/player/Player.java:1974-1980`, local player için sanal
  `getFlyingSpeed` override'ının uçuş yeteneği, sprint ve passenger dallarını
  belirlediğini gösterir.

## Davranış sözleşmesi

- Oyuncu zemindeyse ve block friction `0.6` veya daha düşükse `getSpeed()`
  olduğu gibi döner. Karşılaştırma strict `>` olduğu için eşik değeri scaling
  dalına girmez.
- Zemin friction değeri `0.6` değerinden büyükse sonuç, Java'nın f32 işlem
  sırasıyla `speed * (0.21600002F / (friction * friction * friction))` olur.
- Local player havadaysa, uçuş yeteneği açık ve passenger değilse ability flying
  speed kullanılır; sprint açıkken bu değer ikiyle çarpılır.
- Uçuş dalı kullanılmadığında sprint yapan oyuncu `0.025999999F`, diğer oyuncu
  `0.02F` alır. NaN friction strict karşılaştırmayı geçemez; speed signed-zero
  sonucu da bit düzeyinde korunur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/friction-influenced-speed` kaydı 11 gözlem taşır. Eşik ve eşik
altı, üstü ve hemen üstü friction; normal/sprint/flying/passenger hava dalları;
negative zero speed ve NaN friction kapsanır.

Java oracle, pinned JAR içindeki gerçek private
`getFrictionInfluencedSpeed` metodunu constructor çalıştırmadan ayrılmış
`LocalPlayer` fixture'ı üzerinde reflection ile çağırır. Fixture'ın Player
ability alanı, sprint/passenger ve zemin durumları kontrollü olarak kurulur;
beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`7f6a545b6a322af1734dcbf26e86c4f49052c9650f7f515918d8ac6be5b8a212`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2955** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/voxel-world/src/input.rs` içindeki `local_player_friction_influenced_speed`, zemin
friction hesabını ve `Player.getFlyingSpeed` hava dallarını f32 bitleriyle
uygular. `ClientWorld::apply_local_player_friction_influenced_speed`, sonucu
local player'ın sorgulanabilir `friction_influenced_speed` state'ine yazar;
flying ability, sprint flag ve vehicle state mevcut world kopyasından okunur.
Behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`7b74d001f7e469e379118fa312a9401f16cf63ba0580f0009d186b6ea20458df` ile aynı
capture üzerinde **2955/2955** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`275409c6de0cb79cef3bd9a40cef1dc9e3de4761ef514d20c139d8a12c747e93` ile aynı
capture üzerinde **2955/2955** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi zemin threshold/scaling, flying sprint ve
passenger fallback dallarını sorgulanabilir local player state'inde doğrular.
`cargo test --locked --offline` koşusunda **178** test geçti; format, all-target
check ve warnings-as-errors clippy de geçti.

## Vanilla server-conformance ayrımı

Bu yardımcı yeni serverbound hareket paketi üretmediği için P10.d.2 için yeni
canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı
ölçülebilir login/world tick smoke testi ve ilk movement wire kabulü
[P10.a kaydında](client-player-input-conformance.md) `server-conformance` olarak
tutulur; bu kayıt yalnız Java istemci davranışıyla Rust hız state'i eşitliğini
iddia eder.

## Açık kapsam

Block friction'ın world/block state ve attribute modifier'larından tam
üretilmesi, su/lav/merdiven travel, collision-aware movement, gerçek local
player tick'i ve serverbound movement paketleri sonraki P10 ve P11 dilimlerinde
açıktır.
