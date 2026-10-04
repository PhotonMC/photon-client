# T35e — Nautilus ve zombi nautilus

**Amaç:** `NautilusModel` (yetişkin, bebek), zombi nautilus ve mercanlı çeşidi (çeşit sunucudan). Gardiyan ve elder gardiyan T35h'ye ayrıldı.
**Bağımlılık:** T35b. **Boy:** M.

## Oku
- Java: `client/model/animal/nautilus/*.java`, `client/model/monster/nautilus/*.java`, `client/renderer/entity/{Nautilus,ZombieNautilus}Renderer.java`, `world/entity/animal/nautilus/*.java`.

## Oracle
`axolotl_model` kalıbı (gerçek model); çeşit tablosu `entity/variants`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar; eyer ve gövde zırhı katmanları; gardiyan (T35h).

## Devir notu
- Yapıldı: `setup_nautilus` (gövde ±10° bakış, her zaman çalışan yüzme, mercan gizleme), zombi nautilus çeşit tablosu (oracle'dan), çeşit verisi 21, `crates/voxel/src/nautilus.rs`. Kayıt: `docs/client-nautilus-parity-conformance.md`.
- Bölündü: gardiyan ve elder gardiyan T35h'ye alındı (ışın çizimi ayrı iş).
- Sapma: eyer ve gövde zırhı katmanları yok (zırh yalnız mercanları gizler); doku yolları oracle'sız. Canlı gözlem yapılmadı.
