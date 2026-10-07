# T70b — Partikül motoru ve `LevelParticles` — [client-parity] ve [server-conformance]

Bu kayıt, `ParticleEngine`in (bekleyen kuyruk, grup sınırı ve rezervuar elemesi, tik sırası),
`ClientLevel.addParticle` geçidinin (kameraya 32 blok, `ParticleStatus`, `calculateParticleLevel`)
ve `handleParticleEvent`in (sayısız paket, gauss dağılımlı sayılı paket) Java 26.2 ile Rust'ta aynı
olmasını kapsar. `client-parity`: gerçek motor ve gerçek `ParticleResources` sağlayıcıları gerçek
`ClientLevel`'da çalışır; tik başına bekleyen ve gruptaki sayılar, ilk partiküller ve tüm grubun
özeti (konum, hız, yaş, ömür, sprite) bit bit karşılaştırılır. `server-conformance`: paketin
kablodaki çözümü Rust birim testleriyle sınanır; canlı sunucu gözlemi yoktur (çizim T70c'dedir).

## Java kaynak incelemesi

- `ParticleEngine.tick`: önce grup tiklenir, sonra bu arada eklenenler gruba girer; yeni
  partikül aynı tik oynamaz. `ParticleGroup.add`: 16384'te dolu; 12288'den sonra giriş şansı
  `((16384 - boyut) / 4096)²` ve motorun rastgele kaynağından bir `nextFloat` çeker.
- `ClientLevel.doAddParticle`: sınırlayıcıyı aşmayan istek, kameraya 1024 kare uzaklıktan
  yakın ve seçenek en düşük değilse yapılır; `calculateParticleLevel` her istekte düzeyi seçer
  (`alwaysShow` ve `MINIMAL` iken onda bir `DECREASED`, `DECREASED` iken üçte bir `MINIMAL`).
- `handleParticleEvent`: sayı 0 ise hızlar `maxSpeed * dist` (float çarpımı), değilse her partikül
  için dinleyicinin rastgele kaynağından altı gauss (üç sapma, üç hız).
- `nextGaussian` `Math.log` kullanır; platformun libm'i ile fdlibm'in sonucu seyrek olarak son
  basamakta ayrışır. `voxel_core::math::strict_log` fdlibm'in `log`unu taşır (JVM'in `StrictMath.log`u,
  400 bin Java sonucuyla bit bit denendi) ve gauss kaynağı onu kullanır.

## Kapsam ve dal kararı

`movement/particle-engine` 10 satır: seviyeden ekleme (yakın, uzak, sınırlayıcıyı aşan, 32 blok
sınırının iki yanı), seçenek üç düzey × `alwaysShow`, paket (sayısız, sayılı, `DECREASED`, uzak ve
`override`), rezervuar ve dolu grup (iki kez 14000). Aynı yolu izleyen türler satır değil, aynı kodun
sabitidir: yalnız `smoke`, `flame`, `small_flame` ve `soul_fire_flame`/`copper_fire_flame` sağlayıcıları
taşınmıştır, ötekiler sağlayıcı görevlerinde gelir (sağlayıcısı olmayan tür hiçbir şey yapmaz, Java'da olduğu gibi).

## Sapmalar (açık)

- `ParticleLimit` (sporlar için 1000) ve `TrackingEmitter` taşınmadı; ilk kullanan tür gelince.
- Seçenekli türlerin paketi (blok, toz, eşya…) okunmaz: yük atlanır, partikül yapılmaz.
- Uygulama `ParticleStatus`u hep `ALL` alır: video ekranının `Particles` değeri saklanmıyor.
- Kamera konumu son çizilen karenin konumudur (Java'da `mainCamera().position()`).
