# T70b — Partikül motoru, `LevelParticles` paketi ve ekleme geçidi

**Amaç:** `ParticleEngine` (bekleyen kuyruk, grup başına en çok 16384, 12288'den sonra rezervuar eleme, tik sırası, `clear`), `ClientboundLevelParticlesPacket` (yalın türler; seçenekli türün yükü atlanır), `ClientLevel.addParticle` geçidi (kameraya 32 blok, `ParticleStatus` ve `calculateParticleLevel`), `count == 0` ve gauss dağılımlı `handleParticleEvent`. Üretimde motor oyun döngüsünde tiklenir.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{ParticleEngine,ParticleGroup,QuadParticleGroup,ParticleResources}.java` (`MutableSpriteSet`), `client/multiplayer/{ClientLevel.java (doAddParticle, calculateParticleLevel),ClientPacketListener.java (handleParticleEvent)}`, `core/particles/{ParticleTypes,ParticleLimit}.java`. Rust: `crates/voxel-world/src/particles/`, `crates/voxel-net/src/connection/play/incoming/world.rs` (`on_sound` örnek), `voxel-oracle/src/particles.rs`, `docs/records/client-particle-core-parity-conformance.md`.

## Oracle
`movement` ailesinde `particle_engine` operation'ı: `ParticleEngine` ve gerçek `ParticleResources` sağlayıcılarıyla `createParticle`/`add`/`tick` ve paket işleyici; her partikül `seededParticle` ile tohumlu (yeniden denemede kuyruğa sızan partikül silinir), motor ve `Level.random`/dinleyici rastgele kaynakları `setSeed`. Çıktı: tik başına grup boyutları, bekleyen sayısı ve hayatta kalan partiküllerin durumu.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Çizim (T70c); `TrackingEmitter`; seçenekli partikül türleri; `ParticleLimit` (onu kullanan ilk tür gelince).

## Belgeler
Bu görevin kaydı (`docs/records/` altında yeni dosya) ve `docs/index.md`.

## Devir notu
(boş)
