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
`docs/client-leash-knot-parity-conformance.md` sapma satırını güncelle.

## Devir notu
(boş)
