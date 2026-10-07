# T33d — Enerji girdabı ve şarjlı creeper — [client-parity]

Bu kayıt, `EnergySwirlLayer`'ın tüketicisi `CreeperPowerLayer`'ın (şarjlı creeper) ne
gönderdiğinin ve render türünün hattının (`RenderPipelines.ENERGY_SWIRL`, kayan doku, toplamalı
harman) Java 26.2 ile Rust'ta aynı olmasını kapsar. Kanıt sınıfı `client-parity`'dir (katmanın
gönderdikleri, hat durumu, kayma) ve canlı gözlemdir (çizim): gerçek `CreeperPowerLayer`,
yakalayıcı bir `SubmitNodeCollector` ile çağrılır; çizim sırası, doku, renk, ışık, örtü,
kayma (doku dönüşüm matrisinin öteleme sütunu) ve hat (anahtarlar, değerler, harman, derinlik,
eleme) bit bit karşılaştırılır; `creeper#armor` modeli gerçek `CreeperModel` ile.

## Java kaynak incelemesi

- `EnergySwirlLayer.submit`: yalnız `isPowered` iken, `order(1)` ile kendi modelini (creeper'da
  genişletilmiş `creeper_armor` katmanı) ana modelin pozunda, `-8355712` rengiyle (0xFF808080),
  ışıksız ve örtüsüz gönderir; doku kayması `(t * 0,01 % 1, t * 0,01 % 1)`, `t` kısmi tikli yaş.
- `ENERGY_SWIRL`: entity shader'ı `EMISSIVE`, `NO_OVERLAY`, `NO_CARDINAL_LIGHTING`,
  `APPLY_TEXTURE_MATRIX`, `ALPHA_CUTOUT 0,1`; harman toplamalı (bir, bir); iki yüz; derinlik
  büyük-eşit ve yazılır. Referansta doku tekrarlıdır, kayma bu yüzden sarar.
- `Creeper.DATA_IS_POWERED` (17, boolean).

## Rust

`render_types.rs`: `ENERGY_SWIRL`, `POWER_TEXTURE`, `POWER_COLOUR`, `swirl_offset`. `mob_vertices_swirl`
köşeye dokunun paketlenmiş görüntüdeki yerini (köşe ve boyut: tint), kaymayı (ışık) ve rengi (gölge)
yazar; Vulkan'da `Swirl` hattı (toplamalı harman, derinlik yazar, eleme yok) parça shader'ında
dokuyu kendi dikdörtgeninde sarar: `fract` ile.

## Kapsam

`entity/swirl-layers` 10 satır (beş yaş × şarjlı/değil), `creeper_model`'e `creeper#armor` satırları.

## Sapmalar (açık)

- Sarma GPU'nun tekrar adres kipi yerine shader'da `fract` ile yapılır; doku sınırında yarım
  texel'den küçük farklar olabilir.
- Renk köşede tek bir sabit (`POWER_COLOUR`); başka girdap tüketicileri (wither, ejderha) kendi
  renklerini aynı yoldan verir (T37).
- Şarj çarpması/ses/yıldırımla şarj yok.
