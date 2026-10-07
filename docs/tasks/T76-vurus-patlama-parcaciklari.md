# T76 — Vuruş, patlama ve ifade parçacıkları

**Amaç:** `CritParticle`, `AttackSweepParticle`, `ExplodeParticle`, `HugeExplosionParticle` ve tohumu, `SonicBoomParticle`, `HeartParticle` (kızgın köylü dahil), `NoteParticle`, `ShriekParticle`, `SculkChargeParticle`, `GustParticle`, `ElderGuardianParticle`, `ClientboundExplodePacket`.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{CritParticle,AttackSweepParticle,ExplodeParticle,HugeExplosionParticle,HugeExplosionSeedParticle,SonicBoomParticle,HeartParticle,NoteParticle,ShriekParticle,SculkChargeParticle,GustParticle,ElderGuardianParticle,ElderGuardianParticleGroup}.java`, `network/protocol/game/ClientboundExplodePacket.java`. Rust: T70.

## Oracle
`particle_state` satırları; `Explode` paketinin yarattığı parçacıklar ve ses. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
