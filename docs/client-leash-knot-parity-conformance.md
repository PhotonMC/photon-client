# T40f — Bağ düğümü — [client-parity]

Bu kayıt çitin üstündeki bağ düğümünün (`LeashFenceKnotEntity`, `LeashKnotRenderer`) çizimini kapsar; bağ çizgisi ayrı görevdedir ([T40k](tasks/T40k-bag-cizgisi.md)). Kanıt sınıfı `client-parity`'dir: model gerçek `LeashKnotModel`'e, yerleşim gerçek `LeashKnotRenderer.submit`'in bıraktığı pose'a karşı karşılaştırılır. Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `LeashKnotModel`: tek parça (`knot`, 6×8×6, doku 32×32), animasyonsuz. `LeashKnotRenderer.submit`: yalnız `(-1, -1, 1)` ölçek (bir mob modeli gibi çevrilir), `entity/lead_knot/lead_knot` dokusu, `NO_OVERLAY`.
- Entity konumu sunucunun gönderdiğidir (`blok + (0,5; 0,375; 0,5)`); `shouldRenderAtSqrDistance` 1024'ten uzağı çizmez (çizim mesafesi genelinde karşılanır).

## Kapsam

`entity/leash-knot-model` 1 satır; `entity/small-projectile-pose`'a 2 satır (çit ve uzak konum). Rust: `MobModel::setup_leash_knot`, `projectile::leash_knot_pose`, oyunda `bullets.rs`.

## Sapmalar ve açık kalanlar

- Bağ çizgisi, `SetEntityLink` ve bağlı entity'nin çizgisi yok (T40k). Çit bloğuna bağlı kalma/silinme sunucu işidir.
