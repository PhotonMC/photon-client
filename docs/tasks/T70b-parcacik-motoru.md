# T70b — Partikül motoru, `LevelParticles` paketi ve ekleme geçidi

**Amaç:** `ParticleEngine` (bekleyen kuyruk, grup başına en çok 16384, 12288'den sonra rezervuar eleme, tik sırası, `clear`), `ClientboundLevelParticlesPacket` (yalın türler; seçenekli türün yükü atlanır), `ClientLevel.addParticle` geçidi (kameraya 32 blok, `ParticleStatus` ve `calculateParticleLevel`), `count == 0` ve gauss dağılımlı `handleParticleEvent`. Üretimde motor oyun döngüsünde tiklenir.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{ParticleEngine,ParticleGroup,QuadParticleGroup,ParticleResources}.java` (`MutableSpriteSet`), `client/multiplayer/{ClientLevel.java (doAddParticle, calculateParticleLevel),ClientPacketListener.java (handleParticleEvent)}`, `core/particles/{ParticleTypes,ParticleLimit}.java`. Rust: `crates/photon-world/src/particles/`, `crates/photon-net/src/connection/play/incoming/world.rs` (`on_sound` örnek), `photon-oracle/src/particles.rs`, `docs/records/client-particle-core-parity-conformance.md`.

## Oracle
`movement` ailesinde `particle_engine` operation'ı: `ParticleEngine` ve gerçek `ParticleResources` sağlayıcılarıyla `createParticle`/`add`/`tick` ve paket işleyici; her partikül `seededParticle` ile tohumlu (yeniden denemede kuyruğa sızan partikül silinir), motor ve `Level.random`/dinleyici rastgele kaynakları `setSeed`. Çıktı: tik başına grup boyutları, bekleyen sayısı ve hayatta kalan partiküllerin durumu.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Çizim (T70c); `TrackingEmitter`; seçenekli partikül türleri; `ParticleLimit` (onu kullanan ilk tür gelince).

## Belgeler
Bu görevin kaydı (`docs/records/` altında yeni dosya) ve `docs/index.md`.

## Devir notu
- **Yapılan:** `photon-world/src/particles/{engine,group,resources}.rs`; `photon-net` `read_level_particles_packet` (id 47) ve `take_level_particles`; `photon/src/particles.rs` (motor, saat tohumlu rastgele kaynaklar, `particles/*.json` sprite listeleri; tik, paket, seviye sıfırlama bağlandı). Oracle `particle_engine`; kayıt: `client-particle-engine-parity-conformance.md`.
- **Sapma/yan ürün:** `nextGaussian` artık `strict_log` (fdlibm) kullanır: arm64'te libm `ln` 1 ulp ayrışıyordu.
- **Açık:** `ParticleStatus` seçeneği saklanmıyor (hep `ALL`); sağlayıcılar yalnız duman/alev; `ParticleLimit`, `TrackingEmitter` yok; çizim T70c.
