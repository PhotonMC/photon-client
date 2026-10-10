# T76c2 — Elder gardiyan parçacığı (ekran çakması)

**Amaç:** `ElderGuardianParticle`/`ElderGuardianParticleGroup`: bir elder gardiyana bakınca ekranda
beliren, kameraya göre sabit duran ve döndürülen baygın gardiyan modeli (`ParticleRenderType.
ELDER_GUARDIANS`, kendi render grubu). [T76c](T76c-ciglik-gardiyan.md)'den ayrıldı (çığlık
parçacığı orada bitti); bu kalan kısım entity modelini çizen ayrı bir parçacık render grubu
gerektiriyor.
**Bağımlılık:** T76c, T34c (gardiyan modeli). **Boy:** M.

## Oku
- Java: `client/particle/{ElderGuardianParticle,ElderGuardianParticleGroup}.java`,
  `client/model/monster/guardian/GuardianParticleModel.java`,
  `client/renderer/entity/ElderGuardianRenderer.java` (`GUARDIAN_ELDER_LOCATION`).
- Rust: elder gardiyan modeli zaten T34c/T35h'te var (`crates/photon-render/src/entity/` altında
  gardiyan ailesi); `crates/photon-world/src/particles/{particle.rs,group.rs}` (grup/tür deseni),
  `crates/photon-render/src/particles.rs` (çizim ilkelleri).

## Yapılacaklar
1. Dünya tarafı: `ElderGuardianParticle` tiki zaten varsayılan (yerçekimi 0, ömür 30); yalnız
   `elder_guardian` türünü sağlayıcıya bağla (basit parçacık, seçenek taşımaz).
2. Render tarafı: kameraya bağlı poz (`camera.rotation()` sonra X ekseninde `60°-150°*yaşOranı`
   dönüş, ölçek 0.42553192, öteleme `(0, -0.56, 3.5)`), alfa `0.05+0.5*sin(yaşOranı*π)`; gardiyan
   modelinin baked layer'ı (ELDER_GUARDIAN) ile tam ekran üstü çizim (entity translucent pipeline).
3. Yeni bir parçacık render grubu/geçidi: sahne dünyasına değil kameraya göre çizildiği için mevcut
   `DrawList` yollarından ayrı bir yol gerekebilir (HUD'a yakın, ama 3B model).

## Oracle
`particle-ticks` (dünya tarafı, basit); render için ayrı bir sayısal oracle (poz matrisi, alfa,
renk) tanımlanmalı — `docs/gui-oracle.md`'deki gibi bir "G" tarifine benzeyebilir ya da yeni bir
`particle_quads`-tipi operation.

## Kabul
`CHECK OK`; canlıda bir elder gardiyana bakılıp ekran çakması gözle doğrulanır (`tools/photon.py
shot`).

## Belgeler
`docs/records/client-particle-hit-parity-conformance.md`.

## Devir notu
Yapıldı: `ElderGuardian` durumu ve motor grubu (`photon-world/particles/elder_guardian.rs`, 30 tik), oyun olayı 10 (`show_elder_guardian`: oyuncu konumunda parçacık, parametre 1 ise `entity.elder_guardian.curse`), poz/renk/ışık/doku (`photon-render/entity/elder_guardian_particle.rs`) Java'nın `ElderGuardianParticleGroup`'una karşı `elder_guardian_particle_pose` oracle'ında (`entity` ailesi, 31 satır), kare çiziminde entity listesine karışık (`Particles::elder_guardian_vertices`, `mob_vertices_blended`).
Sapma/açık: canlı gözlem yapılmadı (bir sunucu olayı gerek); dünya tikinin Java karşılığı oracle'da kurulamadı (kurucu `Minecraft` ister), Rust birim testi var; puffer fish olayı (9) ve diğer oyun olayları yok.
