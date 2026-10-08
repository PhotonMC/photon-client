# T53 — İşaret ışığı menüsü ve ekranı (`SetBeacon`)

**Amaç:** `BeaconMenu` (ödeme yuvası), `BeaconScreen` (güç düğmeleri, onay/iptal, efekt simgeleri), `ContainerSetData` ile seviye ve efektler, `ServerboundSetBeaconPacket`; menü türü 9 şimdilik kapatılıyor.
**Bağımlılık:** T50. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/BeaconScreen.java`, `world/inventory/BeaconMenu.java`, `network/protocol/game/ServerboundSetBeaconPacket.java`, `world/level/block/entity/BeaconBlockEntity.java` (yalnız efekt listesi). Rust: `crates/photon-game/src/menus/container.rs` (`MenuKind`, `from_menu_type`), `crates/photon-render/src/gui/screen/container.rs`.

## Oracle
Menü: `movement/container-clicks` (ödeme yuvası kuralı, hızlı taşıma); ekran: `container_screen` beacon (seviye 0–4, seçili efekt, devre dışı düğmeler). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
- Yapıldı: `MenuKind::Beacon` (ödeme yuvası, `quickMoveStack`), `BeaconPanel` (düğmelerin yerleşimi, durumu, tıklaması), `ScreenAction::{SetBeacon, Close}` (+ fare tıklamasıyla kapanınca imleç yakalanır), `encode_set_beacon`/`PlaySession::set_beacon`, `ScreenDraw::{partial, widget_tooltips}`. Kayıt: `docs/records/generic-container-conformance.md` "İşaret ışığı".
- Oracle: `container_screen` `beacon-*` satırları (veri sonradan gelirse ekranın `dataChanged`'i için oracle `data_after` alanı); `container-clicks` 2 satır.
- Sapma: ipucu gecikmesiz; yük eşyaları simgeleri çizilmiyor; gerçek sunucuyla bakılmadı. Açık: efekt kimliğini sunucunun kayıt sırası yerine istemcinin `MOB_EFFECTS` sırasıyla kodluyoruz (aynı sıra, protokol kayıtları aynı).
