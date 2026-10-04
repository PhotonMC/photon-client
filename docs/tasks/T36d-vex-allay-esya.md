# T36d — Vex ve allayın elindeki eşya

**Amaç:** `ItemInHandLayer` ile vex ve allayın tuttuğu eşyayı çizmek: `VexModel.translateToHand` (gövde, kol, 0,55 ölçek, 0,046875/−0,15625/0,078125 kayma) ve `AllayModel.translateToHand` (gövde, `(0, 0,0625, 0,1875)`, kolun eğimi, 0,7 ölçek).
**Bağımlılık:** T36b, T41. **Boy:** S.

## Oku
- Java: `client/model/monster/vex/VexModel.java`, `client/model/animal/allay/AllayModel.java` (`translateToHand`), `client/renderer/entity/layers/ItemInHandLayer.java`.
- Rust: `crates/voxel/src/{monsters,held_items}.rs`, `crates/voxel-render/src/{mob_model,held_item}.rs` (`held_item` el pozu kalıbı: `dolphin_item_pose`, `held_item_pose` oracle'ları).

## Oracle
`held_item_pose` / `dolphin_item_pose` kalıbı: gerçek `translateToHand` ile bir kimlik duruştan çıkan matris.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar.

## Devir notu
- Yapıldı: `vex_hand_pose` ve `allay_hand_pose` (`voxel-render/held_item.rs`), `monsters.rs`'te vex ve allayın her eli için `HeldItem` (solak mobda ana el solda). Kayıt: `docs/client-flier-item-parity-conformance.md`.
- Bulgu: oracle matrisi eşya modelinin köşeye çekilmesini (−0,5 her eksende) içerir; ilk karşılaştırma bu yüzden `0,5`'lik farkla düştü, adaptör `translate(−0,5…)` ekledi (panda ile aynı).
- Sapma: kullanım ve mızrak duruşları yok. Canlı gözlem yapılmadı.
