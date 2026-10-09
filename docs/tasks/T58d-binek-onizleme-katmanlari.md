# T58d — Binek önizlemesinde nautilus ve katmanlar

**Amaç:** `InventoryScreen.extractEntityInInventoryFollowsMouse`'ın mount ekranında çizdiği tam görünüm: nautilus ve zombi nautilus modeli, eyer (`SimpleEquipmentLayer`), at/lama/nautilus zırhı ve lama halısı katmanları. T58c yalnız at ailesi, deve ve lamanın gövdesini (ve at işaretlerini) çizer.
**Bağımlılık:** T58c. **Boy:** M.

## Oku
Java: `client/renderer/entity/{HorseRenderer,LlamaRenderer,NautilusRenderer}.java` ve katmanları (`SimpleEquipmentLayer`, `LlamaDecorLayer`), `client/renderer/entity/state/EquineRenderState`. Rust: `crates/photon/src/mobs/animals/farm_animals.rs` (`gui_preview`), `crates/photon/src/mobs/aquatic/` (nautilus), dünyada mount katmanlarının çizildiği yer (`grep -rn saddle crates/photon/src`).

## Yapılacaklar
`gui_preview`i katmanlarla genişlet (dünya çizimiyle aynı yardımcıları kullan; kopyalama), nautilus için aynı çağrıyı aquatic modülünden ver.

## Oracle
Üçgenler karşılaştırılmaz; katman kararı ve doku seçimi mevcut mob parity kayıtlarına dayanır. Gerekirse `gui/container-screen` mount satırlarına ekipmanlı varlık ekle.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`; canlı gözlem yalnız `tools/photon.py shot` ile bir kez.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` satırı.

## Devir notu
(boş)
