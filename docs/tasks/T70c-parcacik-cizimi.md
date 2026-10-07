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
- **Yapılan:** `voxel-render/src/particles.rs` (`extract`, `quad`), `Quaternion::{rotate_z,transform}`; Vulkan `DrawKind::Particles`/`Run::Particle`/`Pass::Particles` (varlık boru hattı ve dokusu); uygulamada `Particles::{textures,bind_textures,vertices}` (partikül dokuları varlık atlasına). Oracle `particle_quads`; kayıt: `client-particle-quads-parity-conformance.md`.
- **Kabul:** `shot` ile `/particle flame|smoke|small_flame` canlı görüldü; çalıştırma raporuna `particle_packets` ve `most_particles` eklendi. İpucu: istemcinin açılışı ~16 sn sürer, `shot` `--wait` ve betik uzunluğu partiküller canlıyken yakalamaya göre seçilir (ör. `none=44`, `--wait 18`).
- **Açık:** yarı saydam katman ve ayrı partikül atlası; `ParticleStatus` seçeneği; sağlayıcılar (T71–T77).
