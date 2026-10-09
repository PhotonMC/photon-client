# T58 — At, eşek, katır ve nautilus envanteri

**Amaç:** `HorseInventoryMenu`/`NautilusInventoryMenu` (eyer, zırh, sandık yuvaları: `AbstractMountInventoryMenu`), `HorseInventoryScreen`/`NautilusInventoryScreen` (varlık önizlemesi, `InventoryScreen.extractEntityInInventory`), `OpenHorseScreen` paketi; binek yolcusu açar (T44b1).
**Bağımlılık:** T50. **Boy:** L.

## Oku
Java: `client/gui/screens/inventory/{AbstractMountInventoryScreen,HorseInventoryScreen,NautilusInventoryScreen}.java`, `world/inventory/{AbstractMountInventoryMenu,HorseInventoryMenu,NautilusInventoryMenu}.java`, `network/protocol/game/ClientboundMountScreenOpenPacket.java`. Rust: `crates/photon/src/screens.rs`, `crates/photon-render/src/gui/screen/inventory.rs` (varlık önizlemesi).

## Oracle
`container-clicks` (eyer/zırh yuvası kuralları); `container_screen` horse (yuva sayısı, önizleme konumu). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
- Bitti (menü yarısı, `client-parity`): `MountScreenOpen` (paket 41), `MenuKind::Mount` (eyer, zırh, sandık, oyuncu), `ArmorSlot` kuralları (`equippable` + `allowed_entities` + `canUseSlot`), `quickMoveStack`, bağlama kilidi; `container/mount.rs`.
- Veri: `item/equippable_allowed.rs` (izin verilen varlıklar, Java dökümünden tablo, fixture testiyle karşılaştırılır); varlık türü etiketleri `ClientBlockStateRegistry::entity_type_tags`; at `tame` bayrağı ve nautilus `TamableAnimal` bayrağı okunur.
- Oracle: `container-clicks`e 13 mount satırı; Java gerçek `HorseInventoryMenu`/`NautilusInventoryMenu`yü gerçek varlıkla kurar (`MovementMenuOracle.mountMenu`). Bulgu: `canUseSlot` at, iskelet/zombi atı ve lamada hep `true`; eyer kuralı eşek, katır, deve ve nautilusta.
- Açık: ekran ve önizleme (T58b); `isAlive` varsayılır (ölü varlık için menü kapanmaz); yuva içeriğini varlığın ekipmanına yazmak yok (sunucu içeriği gönderir).
- Parity: `movement` yalnız bilinen Q04 farkıyla kırmızı.
