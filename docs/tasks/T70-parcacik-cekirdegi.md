# T70 — Partikül motoru çekirdeği ve `LevelParticles` paketi

**Amaç:** `Particle` tabanı (yaşam, yerçekimi, `move` ile çarpışma, `ParticleEngine.tick` sırası ve grup sınırları), `SingleQuadParticle` bilbord çizimi (Vulkan hattı, `particles/*.json` sprite listeleri: `SpriteSet`), `ClientboundLevelParticlesPacket` ayrıştırması (`SimpleParticleType`; seçenekli türler sonraki görevlerde). İlk tüketici `SmokeParticle`/`FlameParticle`.
**Bağımlılık:** –. **Boy:** L.

## Oku
Java: `client/particle/{Particle,ParticleEngine,ParticleGroup,QuadParticleGroup,NoRenderParticleGroup,SingleQuadParticle,SpriteSet,ParticleDescription,ParticleResources,SmokeParticle,RisingParticle,FlameParticle}.java`, `core/particles/ParticleTypes.java`, `network/protocol/game/ClientboundLevelParticlesPacket.java`. Rust: yeni `crates/voxel-world/src/particles/` (durum) ve `crates/voxel-render` (çizim); rastgele tüketim sırası `voxel_core::random::RandomSource`.

## Oracle
Gerçek `Particle` alt sınıfları sahte `ClientLevel` ile `n` tik: konum, hız, yaşam, boyut, alfa, sprite sırası bit bit (`client-parity`); çizim `particle_quads` operation'ı (köşe, UV, renk, ışık). Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
