# T39d — Rüzgâr yükleri ve ejderha topu — [client-parity]

Bu kayıt `wind_charge`, `breeze_wind_charge` (`WindChargeRenderer`) ve `dragon_fireball` (`DragonFireballRenderer`) çizimini kapsar. Kanıt sınıfı `client-parity`'dir: rüzgâr yükünün modeli gerçek `WindChargeModel`'e, ejderha topunun dörtgeni gerçek `DragonFireballRenderer.submit`'in vertex tüketicisine verdiklerine karşı karşılaştırılır. Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `WindChargeModel`: `bone` altında `wind` (iki küp, başlangıçta `-0,7854` dönüşlü) ve `wind_charge` (4×4×4). `setupAnim` her iki parçanın `yRot`'unu atar (toplamaz): yük `-yaş·16°`, rüzgâr `+yaş·16°`.
- `WindChargeRenderer.submit`: yerleşim yok (model entity konumunda, dönüşsüz); `breezeWind` katmanı, doku `u` boyunca `yaş·0,03 % 1` kayar. Model `entityTranslucent` olarak tanımlıdır ama çizilen tür `breezeWind`tir (breeze rüzgârıyla aynı boru hattı: dünya ışığı, yüz gölgesi yok).
- `DragonFireballRenderer.submit`: 2× ölçek, `camera.orientation` ile kameraya dönük tek dörtgen (`x∈[-0,5; 0,5]`, `y∈[-0,25; 0,75]`), `entityCutout`, blok ışığı her zaman 15 (`getBlockLightLevel`).

## Kapsam

- `entity/wind-charge-model` 7 satır (yaşlar 0 … 100,1); `entity/dragon-fireball-quad` 7 satır (kamera dört yön, eğik, uzak konum, iki ışık değeri).
- Rust: `MobModel::setup_wind_charge`, `mob_vertices_scrolled_posed`, `projectile::{dragon_fireball_quad, dragon_fireball_triangles}`, oyunda `crates/photon/src/objects/charges.rs`.

## Sapmalar ve açık kalanlar

- Rüzgâr dokusunun kayma miktarı (`yaş·0,03 % 1`) oracle'da karşılaştırılmaz: Java'da yalnız render türü argümanıdır; kayma uygulaması breeze'inkiyle ortaktır.
- Canlı: ejderha topu görüldü; sunucu rüzgâr yüklerini birkaç tikte yok ettiğinden canlı görüntü alınamadı.
- Dış çizgi (`outlineColor`, parlayan entity) çizilmez; ses ve partikül yok.
