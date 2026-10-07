# T39 — Mermiler ve fırlatılanlar

**Amaç:** Ok (ve spektral ok), trident, ateş topları, rüzgar yükü, shulker mermisi, lama tükürüğü, kartopu/yumurta/ender incisi/iksir (`ThrownItemRenderer`), havai fişek, olta şamandırası, evoker dişleri, ender gözü.
**Bağımlılık:** T24. **Boy:** M. **Türler:** arrow, spectral_arrow, trident, fireball, small_fireball, dragon_fireball, wind_charge, breeze_wind_charge, shulker_bullet, llama_spit, snowball, egg, ender_pearl, splash_potion, lingering_potion, experience_bottle, firework_rocket, fishing_bobber, evoker_fangs, eye_of_ender.

## Oku
- Java: `client/model/object/projectile/ArrowModel.java`, `client/renderer/entity/ArrowRenderer.java`, `client/renderer/entity/ThrownItemRenderer.java`, `client/renderer/entity/ThrownTridentRenderer.java`, `client/renderer/entity/WindChargeRenderer.java`, `client/renderer/entity/ShulkerBulletRenderer.java`, `client/renderer/entity/FishingHookRenderer.java`, `client/renderer/entity/FireworkEntityRenderer.java`, `client/renderer/entity/EvokerFangsRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Ok ve trident: yönelme (`yRot/xRot` hız vektöründen, `shake`) — istemci tiki T21'de var, çizim yok.
2. `ThrownItemRenderer` (eşya sprite'ı kameraya dönük): `item_entity.rs`'in eşya çizimiyle paylaşılır.
3. Fishing hook: olta çizgisi (`SetEntityLink`/`getHookedIn`), ayrı dal.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Parçacıklar ve sesler, mermi çarpışması (sunucu).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Ok ailesi, fırlatılan eşya ailesi, ateş topu ailesi üç dal; geri kalanı tek tek küçük. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı yalnız ok ve spektral ok: `ArrowModel` (titreme), `arrow_pose`, `AbstractArrow` istemci durumu (`shakeTime`, yerde, iksir rengi), `entityCutoutCull` için CPU'da yüz ayıklama, `arrows.rs` çizici. Kayıt: `docs/records/client-arrow-parity-conformance.md`.
- Bölündü: [T39b](T39b-trident.md) trident, [T39c](T39c-firlatilan-esya.md) fırlatılan eşyalar ve havai fişek, [T39d](T39d-atesler-ruzgar-yukleri.md) ateş topları ve rüzgâr yükleri, [T39e](T39e-shulker-mermisi-tukuruk-disler.md) shulker mermisi, lama tükürüğü, evoker dişleri, [T39f](T39f-olta-samandirasi.md) olta şamandırası.
- Sapma: saplanma çarpışması ve kritik parçacıklar yok; canlı: üç ok `summon` ile görüldü.
