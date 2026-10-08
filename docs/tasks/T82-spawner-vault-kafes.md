# T82 — Spawner, vault ve deneme spawner'ı çizimi

**Amaç:** `SpawnerRenderer` (dönen mob önizlemesi), `TrialSpawnerRenderer`, `VaultRenderer` (dönen eşya), `CampfireRenderer` (pişen eşyalar), `CopperGolemStatueBlockRenderer`.
**Bağımlılık:** T79, T24. **Boy:** M.

## Oku
Java: `client/renderer/blockentity/{SpawnerRenderer,TrialSpawnerRenderer,VaultRenderer,CampfireRenderer,CopperGolemStatueBlockRenderer}.java`. Rust: T79 ve entity çiziciler.

## Oracle
`block_entity_model`: dönüş açıları ve yerleşim. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
