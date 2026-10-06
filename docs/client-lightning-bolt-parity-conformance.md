# T40e — Yıldırım — [client-parity]

Bu kayıt `lightning_bolt` entity'sinin (`LightningBolt`, `LightningBoltRenderer`) çizimini kapsar. Kanıt sınıfı `client-parity`'dir: bir tohum için gerçek `submit`'in vertex tüketicisine verdiği her köşe (konum, renk) Rust çıktısıyla karşılaştırılır. İstemci sayacı birim testlidir; canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `LightningBoltRenderer.submit`: sekiz basamaklı gövdenin kayması `RandomSource(seed)` ile (`nextInt(11) − 5`), dört genişlikte (`0,1 + r·0,2`) üç geçiş: gövde (`h` 7→0, dar kayma) ve iki dal (`7−p` den `5−p`'ye, geniş kayma `nextInt(31) − 15`); her parça dört dörtgen (renk `0.45, 0.45, 0.5, 0.3`). Tohum her dörtgen grubunda yeniden kurulur.
- `LightningBolt`: tohum ve parlama sayısı (`nextInt(3) + 1`) entity'nin tohumsuz rastgelesinden gelir; istemci tiki ömrü (2'den) azaltır; ömür bitince bekleme (`nextInt(10)`) sonra yeni tohumla yeniden parlar, parlama bitince entity silinir.
- Boru hattı `lightning`: eklemeli karışım; ışık ve gölgelendirme yok.

## Kapsam

- `entity/lightning-bolt` 7 satır (sıfır, bir, sıradan, negatif, uç tohumlar, uzak konum): 4 genişlik × 14 parça × 4 dörtgen × 4 köşe.
- Rust: `voxel-render::lightning::lightning_bolt`, `voxel-world::lightning` (tohum, parlama, silinme; birim testli), oyunda `lightning_bolts.rs` (ejderha ışınlarının eklemeli hattından).

## Sapmalar ve açık kalanlar

- Tohum ve parlama sayısı entity'nin kimliğinden türetilen rastgeleden gelir: Java'da tohumsuz olduğundan gözlemlenemez. Bekleme/silinme sayacı gerçek tike karşı karşılaştırılmadı.
- Sesler (`LIGHTNING_BOLT_THUNDER`, `IMPACT`) yok.
- Karışımın derinlik/yüz ayıklama davranışı oracle'da karşılaştırılmaz (ışınların hattı kullanıldı).

## Gökyüzü çakması (T40j) — [client-parity]

- Java: `LightningBolt.tick` istemcide ömür `>= 0` iken `ClientLevel.skyFlashTime`'ı 2 yapar; `ClientLevel.tick` her tikte bir azaltır (`Minecraft.tick` önce `tickEntities`, sonra `level.tick` çalıştırır). `skyFlashTime > 0` iken `SKY_COLOR` `srgbLerp(0,22; renk; 0xFFCCCCFF)`, `SKY_LIGHT_FACTOR` 1,0 olur (`ClientLevel`'ın son iki katmanı); `hideLightningFlashes` seçeneği süreyi sıfır gösterir.
- Rust: `ClientWorld::{tick_entities (ışıklı bolt ise 2), tick_sky_flash, sky_flash_time}`, `Environment::evaluate`'ın `sky_flash` bağımsız değişkeni; seçenek `options.txt`'te (`hideLightningFlashes`, bir ekranda yok).
- Oracle: `movement/sky-flash` 9 satır — gerçek `ClientLevel.environmentAttributes()` ile çakma süresi 0/1/2, seçenek açık/kapalı, üç yükseklik; gökyüzü rengi ve ışık çarpanı bit bit eşit. Yüklü parça, yağmur/fırtına ve saat ilerlemesi (zaman çizelgesi) bu oracle'da yok (bitki örtüsü ovası, saat sıfır).
- Açık: bolt'un `life >= 0` ve süre sayacı gerçek tike karşı karşılaştırılmadı (tohum gibi entity rastgelesi tohumsuz).
