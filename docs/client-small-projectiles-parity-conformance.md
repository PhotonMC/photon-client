# T39e — Shulker mermisi, lama tükürüğü ve evoker dişleri — [client-parity]

Bu kayıt `shulker_bullet`, `llama_spit` ve `evoker_fangs` çizimini kapsar. Kanıt sınıfı `client-parity`'dir: modeller gerçek `ShulkerBulletModel`, `LlamaSpitModel` ve `EvokerFangsModel`'e, yerleşim gerçek `ShulkerBulletRenderer`, `LlamaSpitRenderer` ve `EvokerFangsRenderer.submit`'in bıraktığı pose'lara karşı karşılaştırılır. Canlı gözlem `server-conformance`'tır.

## Java kaynak incelemesi

- `ShulkerBulletModel.setupAnim`: tek parça (`main`, üç çapraz levha), `yRot`/`xRot` mermi yönüdür. Renderer aynı modeli iki kez çizer: çekirdek (`-0,5, -0,5, 0,5` ölçek, kesmeli, tam kalın) ve bunun 1,5 katı kabuk (`entityTranslucent`, renk `0x26FFFFFF`, yani alfa 38/255). Yaşla (`ageInTicks`) üç eksende döner; blok ışığı her zaman 15.
- `LlamaSpitModel`: hareketsiz, yedi 2×2×2 küp. Renderer: `0,15` yukarı, `Y(yaw − 90)`, `Z(eğim)`.
- `EvokerFangsModel.setupAnim`: çeneler `bite` ile kapanır (`1 − min(2·bite, 1)³`), gövde `(bite + sin(2,7·bite))·7,2` batar, `bite > 0,9` iken kök `(1 − bite)/0,1` ölçeğine küçülür (`root.y = 24 − 20·ölçek`). Renderer `bite == 0` iken **hiçbir şey çizmez**; yaw `getYRot()` (aradeğersiz), `Y(90 − yaw)`, `(-1, -1, 1)` ölçek, `-1,501`.
- `EvokerFangs` istemci durumu: olay 4 (`clientSideAttackStarted`) gelene dek durur; sonra her tik `lifeTicks` (22'den) azalır; `getAnimationProgress = 1 − (lifeTicks − 2 − kısmi)/20`, kalan ≤ 0 iken 1.

## Kapsam

- `entity/shulker-bullet-model` 4, `entity/llama-spit-model` 1, `entity/evoker-fangs-model` 10 satır (ısırmanın ucları, 0,9 eşiği); `entity/small-projectile-pose` 20 satır (yaşlar, yönler, ısırma öncesi/sonrası).
- Rust: `MobModel::{setup_shulker_bullet, setup_llama_spit, setup_evoker_fangs}`, `projectile::{shulker_bullet_poses, llama_spit_pose, evoker_fangs_pose}`, `voxel-world::evoker_fangs` (olay ve sayaç; birim testli), oyunda `crates/voxel/src/bullets.rs`.
- Kabuk, entity gölgelendiricisine eklenen yeni bir kodla çizilir: `shade ≥ 64` dünya ışığıyla harmanlanan, alfası `shade − 64` olan köşedir.

## Sapmalar ve açık kalanlar

- Fangs durumunun (olay 4, sayaç) gerçek Java tikine karşı karşılaştırması yok; formül birim testtedir. Kabuğun harmanlama sırası ve alfa uygulaması oracle'da karşılaştırılmaz (canlıda kabuğun saydam görüldüğü doğrulandı).
- İstemci hareket tahmini yok (mermi ve tükürük yalnız sunucunun gönderdiği konumda): [T39g](tasks/T39g-mermi-hareketi.md).
- Dış çizgi, partiküller (`END_ROD`, `CRIT`) ve sesler yok.
