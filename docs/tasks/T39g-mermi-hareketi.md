# T39g — Zarar veren mermilerin istemci hareketi

**Amaç:** `shulker_bullet`, `llama_spit`, `dragon_fireball`, `wind_charge`, `breeze_wind_charge` ve kalan `hurtingprojectile`'ların istemci tikinde konumu ve dönüşü tahmin edilir (sunucu bunları seyrek günceller; şimdi yalnız son paketin konumunda çizilirler).
**Bağımlılık:** T39d, T39e. **Boy:** M.

## Oku
- Java: `world/entity/projectile/{ShulkerBullet,LlamaSpit,ProjectileUtil}.java` (`rotateTowardsMovement`), `hurtingprojectile/{AbstractHurtingProjectile,DragonFireball}.java`, `hurtingprojectile/windcharge/*`.
- Rust: `crates/voxel-world/src/entity.rs` (`VanillaProjectileKind`, `tick_arrow`), `crates/voxel-world/src/lib.rs` (tik dalı).

## Yapılacaklar
Her tür için Java'nın istemci tikini (konum + hareket, sürtünme, yer çekimi, `rotateTowardsMovement`) gerçek varlığın tikine karşı `living-entity-animations` satırıyla karşılaştır. Çarpışma/yok olma (`LlamaSpit` blokta silinir) dünya sorgusu ister: önce okun yol-açıklığı denetimini (`projectile_path_is_open`) kullan.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Partiküller ve sesler.

## Belgeler
`docs/client-small-projectiles-parity-conformance.md` sapma satırını güncelle.

## Devir notu
`flight.rs`: zarar veren mermilerin, shulker mermisinin ve tükürüğün istemci tiki; `projectile_ticks` oracle'ı (MovementOracle) gerçek varlığa karşı bit bit eşit. Kayıt `client-small-projectiles-parity-conformance.md` sonunda.
Ek bulgu: rüzgâr yükünün kutusu 0,15 aşağıda (pick.rs `bounding_box`).
Sapma: onHit, deflection, blok içi etkiler, dünya sınırı, yerel oyuncu hedef/sahip; parçacık/ses yok.
