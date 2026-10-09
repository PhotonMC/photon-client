# T58c — Binek envanteri ekranında varlık modeli

**Amaç:** `AbstractMountInventoryScreen.extractBackground`'ın son adımı: `InventoryScreen.extractEntityInInventoryFollowsMouse(xo+26, yo+18, xo+78, yo+70, 17, 0.25, fare, binek)`. Binek (at, eşek, katır, deve, lama, nautilus) modeli fareye dönük, iki envanter ışığıyla, kutuya sığacak boyda çizilir.
**Bağımlılık:** T58b. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/InventoryScreen.java` (`extractEntityInInventoryFollowsMouse`), `client/gui/GuiGraphicsExtractor.entity`, `client/renderer/entity/state/LivingEntityRenderState.java`. Rust: `crates/photon-render/src/entity/player_renderer.rs` (`gui_player_vertices`, oyuncu için aynı iş), `crates/photon-render/src/gui/screen.rs` (`PlayerDrawer`), `crates/photon/src/mobs/` (binek modelleri ve katmanları: eyer, zırh, halı).

## Yapılacaklar
- `PlayerDrawer` yanına genel bir `EntityDrawer` (varlık kimliği, kutu, boyut, kaydırma, fare) ya da `PlayerDrawer`ı genelle; ev sahibi (`crates/photon`) binek modelini ve katmanlarını GUI üçgenlerine çevirir.
- `GenericScreen::mount_background` sonunda çağır; sprite kayıtları T58b'de bitti.

## Oracle
Java `StubRenderer` modeli kaydetmiyor (draw log'da pip yok); eşitlik `bodyRot`/`yRot`/`xRot` ve çeviri/ölçek hesabının sayısal karşılaştırmasıyla (`gui_player_vertices` gibi) ya da yakalanan `GuiEntityRenderState` alanlarıyla (`x0..y1`, `scale`, `translation`, `rotation`) kurulur; karar burada.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`; canlı gözlem bir kez, `tools/photon.py shot`.

## Belgeler
`docs/gui-oracle.md`, `docs/records/generic-container-conformance.md`.

## Devir notu
(boş)
