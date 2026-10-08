# T73 — Su ve kabarcık parçacıkları

**Amaç:** `BubbleParticle`, `BubbleColumnUpParticle`, `BubblePopParticle`, `SplashParticle`, `WaterDropParticle`, `WakeParticle`, `WaterCurrentDownParticle`, `SquidInkParticle`, `DripParticle` ailesi (damlama, düşme, iniş), yağmur sıçraması (`ClientLevel.tickRain`).
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{BubbleParticle,BubbleColumnUpParticle,BubblePopParticle,SplashParticle,WaterDropParticle,WakeParticle,WaterCurrentDownParticle,SquidInkParticle,DripParticle}.java`, `client/multiplayer/ClientLevel.java` (`tickRain`, `animateTick`). Rust: T70.

## Oracle
`particle_state`: akışkan içinde/dışında kabarcık ömrü, damla durumları (hang/fall/land); `client-parity`. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
Yapılan: `particles/water.rs` (kabarcık, kabarcık sütunu, patlama, yağmur, sıçrama, olta izi, aşağı
akıntı, mürekkep ve parlayan mürekkep; `Kind::Water` alt dalları), sağlayıcı kayıtları.
Oracle: `particle-ticks`'e 20 satır (havuz, zemin, basamak), hepsi bit eşit.
Sapma: mürekkep yarı saydam ama kesik çiziliyor; yalnız kaynak su sahnede.
Açık: `DripParticle` ailesi → T73b.
