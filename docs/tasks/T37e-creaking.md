# T37e — Creaking

**Amaç:** `CreakingModel` (yürüyüş ve saldırı `KeyframeAnimation`'ı, bükülme), `CreakingRenderer` (emissive göz katmanı), `Creaking` istemci durumu (`IS_ACTIVE`, `CAN_MOVE`, `IS_TEARING_DOWN`, ölümde çökme).
**Bağımlılık:** T37, T43b. **Boy:** M.

## Oku
- Java: `world/entity/monster/creaking/Creaking.java`, `client/model/monster/creaking/CreakingModel.java`, `client/renderer/entity/{CreakingRenderer,state/CreakingRenderState}.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs`, `crates/voxel-world/src/animation_state.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_creaking` (bakış, `canMove`'a bağlı yürüyüş, üç animasyon; gövde ve göz katmanı), `voxel-world/creaking.rs` (olay 4/66, sayaçlar, `CAN_MOVE`/`IS_ACTIVE`/`IS_TEARING_DOWN`/`HOME_POS`, animasyon durumları, göz kırpma), `Creaking.updateWalkAnimation` (en çok 3), `monsters.rs`'e creaking ve gözleri (açıkken `glow`). Kayıt: `docs/records/client-creaking-parity-conformance.md`.
- Sapma: göz kırpma zamanlaması karşılaştırılmaz (Java'da tohumsuz rastgele); ses, `tearDown` parçacıkları ve kalp bloğu yok. Canlı gözlem yapılmadı.
- Not: oracle oyuncusunun bağlantısı yok; itilebilir varlık satırlarında oyuncu uzakta tutulmalı (`player_x`).
