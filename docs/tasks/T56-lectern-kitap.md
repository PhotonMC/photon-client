# T56 — Kürsü menüsü ve kitap görünümü

**Amaç:** `LecternMenu` (kitap yuvası, `ContainerButtonClick` ile sayfa/kitabı alma), `LecternScreen` (`BookViewScreen`'i genişletir), `BookViewScreen` (sayfa çizimi, sayfa düğmeleri); yazılabilir kitap düzenleme ve imzalama [PLAN-M6](PLAN-M3d.md).
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `client/gui/screens/inventory/{LecternScreen,BookViewScreen,PageButton}.java`, `world/inventory/LecternMenu.java`. Rust: `crates/voxel-render/src/gui/screen/container.rs`, `crates/voxel-render/src/gui/font.rs`.

## Oracle
`container_screen` lectern + `book_view` operation'ı (sayfa metni sarma, sayfa göstergesi, düğme durumları). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
- Yapıldı: `MenuKind::Lectern`, `BookScreen` (sayfa, numara, düğmeler, tuşlar, `change_page`), `Values::book_pages` (yazılı ve yazılabilir kitap), `Style::shadow` (`withoutShadow`; gölge rengi), `ContainerScreen`ın `may_build` bilgisi. Kayıt: `docs/records/generic-container-conformance.md` "Kürsü ve kitap görünümü".
- Oracle: `container_screen` `lectern-*` 8 satır (gerçek `LecternScreen`; oracle'a `book_pages` eşyası, `userPropertiesFuture`).
- Açık: oyuncunun elindeki kitabı açan `OpenBook` paketi ve kitap düzenleme/imzalama ekranları ([PLAN-M6](PLAN-M3d.md)); `run_command` tıklaması; sayfa sesi.
