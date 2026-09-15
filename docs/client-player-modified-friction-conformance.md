# P10.d.3 — LivingEntity modified block friction — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LivingEntity.computeModifiedFriction` yolunu
block friction ile attribute modifier birleşimi üzerinde karşılaştırır. Kanıt
sınıfı `client-parity`'dir: gerçek Java private static metodu çalıştırılır ve
aynı girdiler production Rust helper'ına verilir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/entity/LivingEntity.java:514-516`, modified friction formülünü ve
  `[0,1]` clamp sınırını tanımlar.
- `world/entity/LivingEntity.java:2470-2474,2491-2494`, bu yardımcı çıktısının
  zemin movement speed ve hava drag yollarına nerede girdiğini gösterir.
- `util/Mth.java:102-108`, f32 clamp'in strict lower-bound ve `Math.min`
  sırasını belirler.

## Davranış sözleşmesi

- Sonuç önce Java f32 işlem sırasıyla `1.0F - (1.0F - friction) * modifier`
  olarak hesaplanır.
- Sonra sonuç `Mth.clamp` ile `[0.0F, 1.0F]` aralığına alınır; alt ve üst
  sınırlar sırasıyla `0.0F` ve `1.0F`'tir.
- Modifier sıfır olduğunda sonuç `1.0F` olur; negatif veya fazla büyük modifier
  değerleri clamp dallarını çalıştırır. NaN friction strict karşılaştırmayı
  geçmez ve Java'nın NaN sonucunu korur.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`player-movement/modified-friction` kaydı sekiz gözlem taşır. Unit, zero,
double, negative ve büyük modifier; zero/full friction ve NaN friction dalları
kapsanır.

Java oracle, pinned JAR içindeki gerçek private static
`computeModifiedFriction` metodunu reflection ile doğrudan çağırır; fixture veya
Rust çıktısından türetilmiş beklenen değer kullanılmaz.

Java referans kimliği:
`22c845e05c0d64d3c4c2bf24af557f639bb18fe2b05e9901f82a82ac57e4d32d`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2956** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`src/input.rs` içindeki `local_player_modified_friction`, formülü Java'nın f32
clamp semantiğiyle uygular. `ClientWorld::apply_local_player_modified_friction`
sonucu local player'ın sorgulanabilir `modified_friction` state'ine yazar;
behavior adapter aynı production helper'ını çağırır.

Debug adapter SHA-256 değeri
`b6ca56fe6168f06b21c7a8795b886fc1990597201ad9fe4fff725e918ec233b1` ile aynı
capture üzerinde **2956/2956** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`5dc2010dca0ced450a351f8981cca659a9860fae38dcb0762f3de50e9939f7c5` ile aynı
capture üzerinde **2956/2956** eşleşti; fark `null` kaldı.

`ClientWorld` regresyon testi formülü, üst-alt clamp ve sorgulanabilir local
player state'ini doğrular. Tam Rust test sonucu ve statik kontroller bu kaydın
commit doğrulamasında tutulur.

## Vanilla server-conformance ayrımı

Bu yardımcı yeni serverbound hareket paketi üretmediği için P10.d.3 için yeni
canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna karşı
ölçülebilir login/world tick smoke testi ve ilk movement wire kabulü
[P10.a kaydında](client-player-input-conformance.md) `server-conformance` olarak
tutulur; bu kayıt yalnız Java istemci davranışıyla Rust friction state'i
eşitliğini iddia eder.

## Açık kapsam

Block friction'ın ilgili block state ve attribute modifier'larından tam
üretilmesi, bu değerin full `travelInAir` akışına bağlanması, su/lav/merdiven
travel, collision-aware movement, gerçek local player tick'i ve serverbound
movement paketleri sonraki P10 ve P11 dilimlerinde açıktır.
