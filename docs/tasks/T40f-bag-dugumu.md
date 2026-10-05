# T40f — Bağ düğümü

**Amaç:** `LeashFenceKnotEntity` (`LeashKnotRenderer`, `LeashKnotModel`) ve `SetEntityLink` ile bağ çizgisi (`EntityRenderer.renderLeash`).
**Bağımlılık:** T40. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{LeashKnotRenderer,EntityRenderer}.java`, `client/model/object/leash/LeashKnotModel.java`, `world/entity/decoration/LeashFenceKnotEntity.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/client-arrow-parity-conformance.md`, `docs/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
(boş)
