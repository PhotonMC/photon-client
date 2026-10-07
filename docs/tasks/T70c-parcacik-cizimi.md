# T70c — Partikül dörtgenlerinin çizimi

**Amaç:** `SingleQuadParticle.extract` (kameraya dönük döndürme, `roll`, kısmi tikte konum), `QuadParticleGroup` kırpma (`frustum.pointInFrustum`), `particles` atlası (`atlases/particles.json`, `particle/` dizini) ve `OPAQUE_PARTICLE`/`TRANSLUCENT_PARTICLE` boru hatları (Vulkan); `particles/*.json` sprite listeleri.
**Bağımlılık:** T70b. **Boy:** L.

## Oku
Java: `client/particle/{SingleQuadParticle,QuadParticleGroup}.java`, `client/renderer/state/level/QuadParticleRenderState.java`, `client/renderer/{RenderPipelines (PARTICLE),feature/ParticleFeatureRenderer}.java`. Rust: `crates/voxel-render/src/texture/atlas.rs` (atlas kurulumu), `render_types.rs` (`RenderType` tablosu), önceki kayıtlar.

## Oracle
`particle_quads` operation'ı: dörtgenin köşeleri, UV, renk ve ışığı (`client-parity`); canlı gözlem (`shot`) yalnız Kabul'de bir kez.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; torçun yanında alev ve duman gözle görünür.

## Kapsam dışı
Başka partikül sınıfları (T71–T77).

## Belgeler
Bu görevin kaydı ve `docs/index.md`.

## Devir notu
(boş)
