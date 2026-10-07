# T74 — Toz, büyü ve renkli parçacıklar

**Amaç:** `DustParticle`, `DustColorTransitionParticle` (`DustParticleOptions`), `FallingDustParticle`, `SpellParticle` (efekt/anlık büyü, `ColorParticleOption`), `TotemParticle`, `GlowParticle` ailesi, `FireworkParticles` (kıvılcım, parlama, başlatıcı).
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{DustParticleBase,DustParticle,DustColorTransitionParticle,FallingDustParticle,SpellParticle,TotemParticle,GlowParticle,FireworkParticles}.java`, `core/particles/{DustParticleOptions,DustColorTransitionOptions,ColorParticleOption}.java`. Rust: T70.

## Oracle
Seçenekli paket çözümü (`wire`) ve `particle_state`: renk ve boyut dönüşümleri. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
