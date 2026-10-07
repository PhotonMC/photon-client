# T72 — Yükselen duman/ateş aileleri

**Amaç:** `RisingParticle` ailesi (`FlameParticle`, `LargeSmokeParticle`, `SoulParticle`, `LavaParticle`, `CampfireSmokeParticle`, `WhiteSmokeParticle`, `PlayerCloudParticle`) ve `BaseAshSmokeParticle` ailesi (`AshParticle`, `WhiteAshParticle`, `DustPlumeParticle`): her sınıfın kendi tik/hareket kuralı.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{RisingParticle,FlameParticle,LargeSmokeParticle,SoulParticle,LavaParticle,CampfireSmokeParticle,WhiteSmokeParticle,PlayerCloudParticle,BaseAshSmokeParticle,AshParticle,WhiteAshParticle,DustPlumeParticle}.java`. Rust: T70'in motoru.

## Oracle
`particle_state` satırları, her sınıf için ilk tikler ve ömür sonu (gerçek sınıflar). Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
