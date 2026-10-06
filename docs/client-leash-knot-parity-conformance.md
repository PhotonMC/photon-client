# T40f — Bağ düğümü — [client-parity]

Bu kayıt çitin üstündeki bağ düğümünün (`LeashFenceKnotEntity`, `LeashKnotRenderer`) çizimini kapsar; bağ çizgisi ayrı görevdedir ([T40k](tasks/T40k-bag-cizgisi.md)). Kanıt sınıfı `client-parity`'dir: model gerçek `LeashKnotModel`'e, yerleşim gerçek `LeashKnotRenderer.submit`'in bıraktığı pose'a karşı karşılaştırılır. Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `LeashKnotModel`: tek parça (`knot`, 6×8×6, doku 32×32), animasyonsuz. `LeashKnotRenderer.submit`: yalnız `(-1, -1, 1)` ölçek (bir mob modeli gibi çevrilir), `entity/lead_knot/lead_knot` dokusu, `NO_OVERLAY`.
- Paket bloğu taşır; istemci `BlockAttachedEntity.setPos` ile konumu `blok + (0,5; 0,375; 0,5)` yapar (T40h'de düzeltildi); `shouldRenderAtSqrDistance` 1024'ten uzağı çizmez (çizim mesafesi genelinde karşılanır).

## Kapsam

`entity/leash-knot-model` 1 satır; `entity/small-projectile-pose`'a 2 satır (çit ve uzak konum). Rust: `MobModel::setup_leash_knot`, `projectile::leash_knot_pose`, oyunda `bullets.rs`.

## Sapmalar ve açık kalanlar

- Çit bloğuna bağlı kalma/silinme sunucu işidir. Bağ çizgisi aşağıdadır.

## Bağ çizgisi (T40k) — [client-parity]

- Java: `ClientboundSetEntityLinkPacket` (iki `int`: bağlı entity, tutucu; 0 bırakmak) `setDelayedLeashHolderId`'ı çağırır; tutucu çizim anında kimlikten bulunur. `EntityRenderer.extractRenderState` bağlı entity için durum kurar: bağlanma noktası (`getLeashOffset`, entity'nin gövde dönüşüyle `yRot`), başlangıç = konum + nokta, bitiş = tutucunun `getRopeHoldPosition`'ı, iki uçta `BlockPos.containing(göz)`'deki blok ve gökyüzü ışığı; hem tutucu hem bağlı dörtlü bağlantıyı destekliyorsa (ghast/happy ghast tutucu; at, deve, tekne, sniffer bağlı) dört durum (`slack = false`). `LeashFeatureRenderer`: 24 adımlı şerit, önce 0,05 yukarıda ileri, sonra geri; ışık uçlar arasında doğrusal; renk `0,5/0,4/0,3` ve 0,7 ikinci tonu.
- Rust: `voxel-net` paketi çözer (`parse_set_entity_link_packet`, kimlik 100) ve `ClientWorld::set_leash_holder` yalnız bağlanabilir türlere işler; tür tablosu oracle'dan (`entity_picks.rs`'e `leash`/`quad_holder`), bağlanma noktası formülleri `voxel-data::leash` (her türün kendi çarpanları; deve oturma/kalkma animasyonuyla), şerit `voxel-render::leash`, durumlar ve çizim `crates/voxel/src/leashes.rs` (dünya metni hattı: düz renk, köşe başına ışık haritası).
- Oracle: `movement/entity-pick-table` her tür için (yetişkin ve yavru) bağlanabilir mi, bağlanma noktası, dörtlü bağlantı noktaları ve tutucu noktalarını bit bit karşılaştırır; `entity/leash-strip` 18 satır gerçek `addVertexPair` köşelerini (gergin/gevşek, uzun/kısa, farklı ışık, ofset) karşılaştırır. Canlı: `summon horse` + çite `data merge ... leash:[I;x,y,z]` ile çizgi görüldü.
- Açık: tutucu elytra ile süzülürken veya mızrakla dönerken (`getRopeHoldPosition`'ın diğer dalı), yerel oyuncunun `xRotO`'su (şimdiki eğim iki yerde), yanan/parlayan entity'nin ışığı (`getBlockLightLevel` geçersiz kılmaları), devenin oturan boyutu, yolcu kayması; hamle: bağlı entity'nin çizgisi hedeflerini gören elle kıyaslanmadı. Köşe sırası, çizim sırası ve hat farkı (derinlik, sis) Java'ya karşı karşılaştırılmadı.
