# T40e — Yıldırım

**Amaç:** `LightningBolt` (`LightningBoltRenderer`): tohumlu rastgele çatallı geometri, `lightning` boru hattı; istemci tarafı gökyüzü aydınlığı ayrı.
**Bağımlılık:** T40. **Boy:** S.

## Oku
- Java: `client/renderer/entity/LightningBoltRenderer.java`, `world/entity/LightningBolt.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/records/client-arrow-parity-conformance.md`, `docs/records/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `lightning_bolt` (geometri, oracle'da köşe köşe), `voxel-world::lightning` (tohum, parlama, silinme), `lightning_bolts.rs` (eklemeli hat). Kayıt: `docs/records/client-lightning-bolt-parity-conformance.md`.
- Sapma: tohum entity kimliğinden; sayaç gerçek tike karşı karşılaştırılmadı; gökyüzü çakması yok → T40j.
