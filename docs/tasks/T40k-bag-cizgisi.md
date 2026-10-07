# T40k — Bağ çizgisi

**Amaç:** `Leashable` entity'den tutucuya (oyuncu, bağ düğümü) uzanan çizgi: `ClientboundSetEntityLinkPacket` ile tutucu kimliği, `EntityRenderer.extractRenderState`'in `LeashState`'i (`getLeashOffset`, `getRopeHoldPosition`, `getPreciseBodyRotation`, dört noktalı bağ: happy ghast), `LeashFeatureRenderer` (24 adımlı şerit, renk `0.5/0.4/0.3` ve `0.7` ikinci tonu, sarkma `slack`).
**Bağımlılık:** T40f. **Boy:** M.

## Oku
- Java: `client/renderer/entity/EntityRenderer.java` (satır ~180–225), `client/renderer/feature/LeashFeatureRenderer.java`, `world/entity/Leashable.java`, `world/entity/Entity.java` (`getRopeHoldPosition`, `getLeashOffset`, `getQuadLeashHolderOffsets`), `LeashFenceKnotEntity.java`.
- Rust: `crates/voxel/src/overlays.rs` (dünya metni hattı: `uv.x < 0` düz renk), `crates/voxel-net/src/connection/entities.rs`, `crates/voxel-render/src/fishing.rs` (çizgi geometrisi kalıbı).

## Yapılacaklar
`SetEntityLink` paketini çöz, her entity'de tutucu kimliğini sakla; çizgiyi dünya metni hattının düz renkli köşeleriyle çiz (ayrı hat gerekirse `leash` render türünün boru hattını kur); gerçek `LeashFeatureRenderer`'ın köşelerini oracle'da karşılaştır.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon` + `data merge entity ... leash`.

## Kapsam dışı
Sesler, bağın kopması (sunucu işi).

## Belgeler
`docs/records/client-leash-knot-parity-conformance.md` sapma satırını güncelle.

## Devir notu
- `SetEntityLink` (kimlik 100) → `ClientWorld::set_leash_holder`; bağlanabilirlik, dörtlü bağlantı ve dörtlü tutucu bayrakları `entity_picks.rs`'e (oracle'dan), bağlanma noktası formülleri `voxel-data::leash` (yetişkin/yavru, deve animasyonu dahil) — pick tablosu oracle'ı her tür için bit bit karşılaştırır.
- `voxel-render::leash` (şerit) 18 satırlık `entity/leash-strip` oracle'ına karşı; `crates/voxel/src/leashes.rs` durumları kurar ve dünya metni hattına verir (canlı: at + çit düğümü görüldü).
- Açık: elytra/mızrak tutucu dalı, yerel oyuncunun xRotO'su, `getBlockLightLevel` geçersiz kılmaları, devenin oturan boyutu, paketi sonradan gelen tutucu için latch (Java'da tutucu bir kez bulununca sabitlenir), çizim hattı farkları Java'ya karşı karşılaştırılmadı.
