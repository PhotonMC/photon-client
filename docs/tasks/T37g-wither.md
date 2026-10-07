# T37g — Wither

**Amaç:** `WitherBossModel` (üç baş, kuyruk, omurga), `WitherBossRenderer` (hasarda mavi/beyaz kaplama, zırh kabuğu), `WitherArmorLayer` (enerji girdabı), `WitherBoss` istemci durumu (`DATA_TARGET_*`, `DATA_ID_INV`, kalkan fazı).
**Bağımlılık:** T37, T33d (enerji girdabı). **Boy:** M.

## Oku
- Java: `world/entity/boss/wither/WitherBoss.java`, `client/model/monster/wither/WitherBossModel.java`, `client/renderer/entity/{WitherBossRenderer,layers/WitherArmorLayer,state/WitherRenderState}.java`.
- Rust: `crates/voxel/src/mobs.rs` (enerji girdabı kalıbı: şarjlı creeper), `crates/voxel-render/src/mob_model.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_wither` (gövde ve zırh katmanı), `voxel-world/wither.rs` (veri 16–19, yan başların hedefe dönüşü, `rotlerp`), `tick_wither_motion` (hareketle yüz çevirme), `monsters.rs`'e wither (ölçek, ışık 15, dokunulmazlık dokusu, can yarıya inince zırh kaydırması). Kayıt: `docs/records/client-wither-parity-conformance.md`.
- Hata dersi: `atan2·180F/(float)PI` javac'ta tek float sabitine katlanır; kaynak gruplaması yanıltır (1 ulp).
- Sapma: parçacık, ses, davranış yok; hedef yalnız canlı varlıklar; canlı gözlem yapılmadı.
