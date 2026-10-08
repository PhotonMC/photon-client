# T40f — Bağ düğümü (yalnız düğüm)

**Amaç:** `LeashFenceKnotEntity` (`LeashKnotRenderer`, `LeashKnotModel`) ve `SetEntityLink` ile bağ çizgisi (`EntityRenderer.renderLeash`).
**Bağımlılık:** T40. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{LeashKnotRenderer,EntityRenderer}.java`, `client/model/object/leash/LeashKnotModel.java`, `world/entity/decoration/LeashFenceKnotEntity.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/records/client-arrow-parity-conformance.md`, `docs/records/client-minecart-parity-conformance.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: düğümün modeli ve pose'u (`bullets.rs`). Kayıt: `docs/records/client-leash-knot-parity-conformance.md`. Bağ çizgisi ve `SetEntityLink` T40k'ye ayrıldı.
