# T58b — Binek envanteri ekranı

**Amaç:** `HorseInventoryScreen`/`NautilusInventoryScreen` (`AbstractMountInventoryScreen`): arka plan (`container/horse`, `container/nautilus`), sandık yuvaları sprite'ı, eyer/zırh yuvası çerçeveleri, boş yuva simgeleri, varlık önizlemesi (`InventoryScreen.extractEntityInInventoryFollowsMouse`) ve başlıkta varlığın görünen adı. Menü ve paket T58'de bitti.
**Bağımlılık:** T58. **Boy:** L.

## Oku
Java: `client/gui/screens/inventory/{AbstractMountInventoryScreen,HorseInventoryScreen,NautilusInventoryScreen,InventoryScreen}.java`, `world/inventory/{HorseInventoryMenu,NautilusInventoryMenu}.java` (`isActive`, boş yuva simgeleri). Rust: `crates/photon-render/src/gui/screen/generic.rs` (`slot_position` mount kolu hazır), `generic/transform.rs` (`texture`, `empty_icon`), `crates/photon-game/src/menus/container/mount.rs`, `crates/photon-net/src/connection/play/incoming/containers.rs` (`on_mount_screen_open`: başlık şimdilik `entity.minecraft.<ad>` çeviri anahtarı).

## Yapılacaklar
- `isActive` kuralı ekranda: eyer yuvası `canUseSlot(SADDLE)` ve `can_equip_saddle` etiketi, zırh yuvası `canUseSlot(BODY)` ve (`can_wear_horse_armor` ya da lama); nautilus yalnız `canUseSlot`. `MountEntity` bu iki bayrağı zaten taşır; etiket bilgisi (etkin yuva) eklenecek.
- Başlık: varlığın özel adı ya da çeviri adı (`Entity.getDisplayName`).
- Varlık önizlemesi için mevcut `InventoryScreen` yerel oyuncu önizlemesini genelle.

## Oracle
G tarifi (README): `GuiOracle.java`'ya `horse_inventory`/`nautilus_inventory` operation'ı (gerçek ekran, `MovementMenuOracle.mountMenu` ile aynı varlık kurulumu), `docs/gui-oracle.md` satırı.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`; canlı gözlem yalnız `tools/photon.py shot` ile bir kez.

## Belgeler
`docs/gui-oracle.md`, `docs/records/generic-container-conformance.md`, `docs/remaining-work.md` §3.

## Devir notu
- Bitti (`client-parity`, gerçek `HorseInventoryScreen`/`NautilusInventoryScreen`): arka plan dokusu, sandık yuvası sprite'ı (kısmi blit), kullanımdaki eyer/zırh çerçeveleri, boş yuva simgeleri, etkin olmayan yuvanın çizilmemesi ve tıklanmaması (`ContainerMenu::slot_active`), başlık. `MountEntity::active` `isActive` kuralıdır.
- Canlı akış: mount menüsü "desteklenmeyen" sayılıp hemen kapanıyordu; `menu_kind` (oturum) artık mount'a duyarlı.
- Oracle: `container_screen`e 12 mount satırı; Java'da `StubRenderer` (varlığın yalnız boyu) ve ölçek tahtası; model kaydedilmez.
- Açık: varlık modeli (T58c); başlık yalnız çeviri anahtarı (özel ad yok).
