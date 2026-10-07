# T57 — Crafter menüsü ve ekranı

**Amaç:** `CrafterMenu` (3×3 ızgara, devre dışı yuvalar `CrafterSlot`, `ContainerSetData` ile yuva kilitleri ve `ServerboundContainerSlotStateChanged`), `CrafterScreen` (kilit simgeleri, kırmızı ok); menü türü 7.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/CrafterScreen.java`, `world/inventory/{CrafterMenu,CrafterSlot,NonInteractiveResultSlot}.java`, `network/protocol/game/ServerboundContainerSlotStateChangedPacket.java`. Rust: `crates/voxel-game/src/menus/container.rs`.

## Oracle
`container-clicks` (kilitli yuvaya koyma, `SlotStateChanged` baytı); `container_screen` crafter (kilit simgeleri, tetiklenmiş ok). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
- Yapıldı: `MenuKind::Crafter` (46 yuva: sonuç yuvası envanterden sonra), yuva kuralları ve `quickMoveStack`, ekran (kapalı yuva, redstone oku, ipucu), yuva açıp kapama (`ScreenAction::SlotState`, `PlaySession::set_crafter_slot_state`, `encode_container_slot_state`). Kayıt: `docs/records/generic-container-conformance.md` "Crafter".
- Oracle: `container-clicks` 3, `container_screen` 6 satır. Sapma: `UI_BUTTON_CLICK` sesi yok; izleyici kipi (spectator) bilinmediğinden tıklamada denetlenmez. Açık: tarif sonucu sunucudan gelir (istemci tahmini yok).
