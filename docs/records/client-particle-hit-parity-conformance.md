# T76/T76c — Vuruş, patlama, ifade ve çığlık parçacıkları — [client-parity]

Bu kayıt vuruş, patlama ve ifade parçacıklarını kapsar: `CritParticle` (`crit`, `damage_indicator`,
`enchanted_hit`), `AttackSweepParticle`, `ExplodeParticle` (`poof`), `HugeExplosionParticle`,
`SonicBoomParticle`, `HeartParticle` (`heart`, `angry_villager`), `NoteParticle`, `GustParticle`
(`gust`, `small_gust`), `SculkChargeParticle`, `SculkChargePopParticle` ve çizilmeyen
yayıcılar `HugeExplosionSeedParticle` (`explosion_emitter`), `GustSeedParticle`
(`gust_emitter_large`, `gust_emitter_small`). Kanıt sınıfı `client-parity`'dir: sağlayıcılar
`movement/particle-ticks`'te tohumlanmış rastgelelerle tek tek koşar, yayıcılar
`movement/particle-engine`'de motorun içinden seviyeye parçacık bırakır. Alanlar bit bit
karşılaştırılır.

## Java kaynak incelemesi

- Kıvılcım: itiş ×0.1, sonra verilen itişin 0.4'ü eklenir; sürtünme 0.7, yerçekimi 0.5; gri
  `r*0.3+0.6`, boy ×0.75, ömür `max(int(6/(r*0.8+0.6)), 1)`, fiziksiz. Kurucu bir kez tikler
  (yeşil ×0.96, mavi ×0.9). Hasar göstergesi dikeyde +1 ile doğar, ömrü 20. Büyülü vuruşta
  kırmızı ×0.3, yeşil ×0.8. Boy ilk 1/32 ömürde büyür.
- Süpürme, patlama, sonik patlama, rüzgâr: durur, tam parlak, yaşla resim değiştirir. Süpürme
  ömrü 4 ve boyu `1−size*0.5`. Patlamanın ömrü `6+nextInt(4)`, boyu `2(1−size*0.5)`. Sonik
  patlamanın ömrü 16, boyu 1.5. Rüzgârın ömrü `12+nextInt(4)`, kutusu 1, küçüğü ×0.15.
- Puf: kendi rastgelesinden ±0.05 saçılır, yerçekimi −0.1, sürtünme 0.9.
- Kalp ve nota: itiş ×0.01, yükseliş 0.1/0.2, sürtünme 0.86/0.66, boy ×1.5, ömür 16/6. Kızgın
  köylü yarım blok yukarıda doğar. Notanın rengi `Mth.sin` tablosuyla itişin x'inden gelir.
- Sculk yükü: sürtünme 0.96, ölçek 1.5 (patlaması 1), fiziksiz, hız verilen itiş, dönüş
  seçenekten, ömür sağlayıcı rastgelesinden; blok ışığı 15, yarı saydam.
- Yayıcılar `NO_RENDER` grubundadır (kendi 16384'lük kapasitesi). Patlama yayıcısı 8 tik boyunca
  her tik 4 blok içinde 6 patlama bırakır. Rüzgâr yayıcısı `age % (delay+1) == 0` tiklerinde 3
  rüzgâr bırakır. İtişin x'i `age/lifetime`'dır.
- `ShriekParticle`: gecikmesi sıfırın üstündeyken hiçbir şey yapmaz (yaş artmaz, taşınmaz, fizik
  yok — yalnız gecikme azalır); sıfıra inince normal tike düşer (yerçekimi 0, sabit yukarı hız
  0,1, ömür 30, boy 0,85). `getQuadSize`/`getLightCoords` örtmeleri (boy `*clamp((yaş+p)/ömür*0,75,
  0,1)`, blok ışığı sabit 15) render tarafında.

## Kapsam ve dal kararı

`particle-ticks`'e 14+2 satır (çığlık gecikmeli/gecikmesiz), `particle-engine`'e yayıcı satırı.
Java gruplarını kimlik hash sırasıyla tikler; oracle her grubu ayrı sayar (`particles`,
`emitters`). Rust önce dörtgenleri, sonra yayıcıları tikler.

## Sapmalar (açık)

- `ElderGuardianParticle` ve `Explode` paketi ayrı: paket T76b'de, `ElderGuardianParticle`
  [T76c2](../tasks/T76c2-elder-gardiyan.md)'de (kameraya bağlı model çizimi, ayrı render grubu
  gerektiriyor — bu kayıt yalnız dünya-tarafı quad parçacıklarını kapsıyor).
- Çığlığın çizimi (iki döndürülmüş dörtgen, render'da hesaplanan alfa) bağlanmadı; dünya tarafı
  tiki (gecikme, yaş, ömür, ışık, boy) kanıtlı.
- Sculk yükünün yarı saydamlığı kesik çiziliyor (T75 ile aynı).
