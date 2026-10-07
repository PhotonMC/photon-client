# T75 — Portal, büyü masası ve çeken parçacıklar

**Amaç:** `PortalParticle`, `ReversePortalParticle`, `FlyTowardsPositionParticle` (enchant, nautilus, vault bağlantısı), `FlyStraightTowardsParticle`, `VibrationSignalParticle`, `TrailParticle`, `SuspendedParticle`/`SuspendedTownParticle`.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{PortalParticle,ReversePortalParticle,FlyTowardsPositionParticle,FlyStraightTowardsParticle,VibrationSignalParticle,TrailParticle,SuspendedParticle,SuspendedTownParticle}.java`, `core/particles/{VibrationParticleOption,TrailParticleOption}.java`. Rust: T70.

## Oracle
`particle_state`; titreşim ve iz paketlerinin seçenek çözümü. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
