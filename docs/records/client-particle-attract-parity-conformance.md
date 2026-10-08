# T75/T75b — Portal, çekim, asılı, titreşim ve iz parçacıkları — [client-parity]

Bu kayıt bir yere çekilen, havada asılı kalan ve bir hedefe doğru kapanan parçacıkları kapsar:
`PortalParticle`, `ReversePortalParticle`, `FlyTowardsPositionParticle` (`enchant`, `nautilus`,
`vault_connection`), `FlyStraightTowardsParticle` (`ominous_spawning`), `SuspendedParticle`
(`underwater`, `warped_spore`, `crimson_spore`, `spore_blossom_air`), `SuspendedTownParticle`
(`mycelium`, `happy_villager`, `egg_crack`, `composter`, `dolphin`), `VibrationSignalParticle`
(`vibration`) ve `TrailParticle` (`trail`). Kanıt sınıfı `client-parity`'dir: gerçek sağlayıcılar
tohumlanmış parçacık, motor ve seviye rastgeleleriyle `movement/particle-ticks`'te koşturulur,
alanlar bit bit karşılaştırılır.

## Java kaynak incelemesi

- Portal: başlangıçtan itişe doğru `1−(−p+2p²)` ile, dikeyde `1−p` eklenerek; boy
  `0.1*(r*0.2+0.5)`, renk `br*(0.9, 0.3, 1)`, ömür `int(r*10)+40`; boy `1−(1−s)²`; ışık yaşın
  dördüncü kuvvetiyle parlar. Ters portal: boy ×1.5, ömür `int(r*2)+60`, her tik itiş×`yaş/ömür`.
- `FlyTowardsPosition`: itilen yerde başlar, başladığı yere `1−p` ile döner, dikeyde `p⁴*1.2`
  iner; boy `0.1*(r*0.5+0.2)`, renk `br*(0.9, 0.9, 1)`, ömür `int(r*10)+30`. Kasa bağlantısı
  parlar, `LifetimeAlpha(0, 0.6, 0.25, 1)` ile çizimde belirir, ×1.5.
- `FlyStraightTowards`: düz geri, ömür `int(r*5)+25`, renk `srgbLerp(p, −12210434, −1)`, opaklık
  dahil; ölçek seviye rastgelesinden `r*2+3`; ışık 15.
- `Suspended`: konum ⅛ blok aşağı, kutu 0.01, sürtünme 1, yerçekimi 0, fiziksiz; sağlayıcılar hız,
  renk ve ömrü sağlayıcının rastgelesinden alır (resimden önce). Spor çiçeği havası
  `ParticleLimit.SPORE_BLOSSOM`'a (1000) sayılır.
- `SuspendedTown`: hız ×0.02, ömrü geri sayar, bloklardan geçer, ×0.99; kompostçu ömrü seviye
  rastgelesinden, yunusunki opaklık ve yarı ömür.
- `VibrationSignalParticle`: `PositionSource` (blok ya da izlenen entity, dikey kaymasıyla)
  hedefine her tikte `1/kalanTik` payı kadar yaklaşır, `Mth.atan2` ile hedefe döner (`rot`/`pitch`,
  çizimde iki dörtgen); hedef bulunamazsa (entity gitmiş) silinir. Işığı `LightCoordsUtil.withBlock(
  ışık, 15)`: blok kanalı sabit 15, gök kanalı normal. `VibrationParticleOption`'ın
  `PositionSource.STREAM_CODEC`'i bir kayıt kimliği (`BLOCK`=0, `ENTITY`=1, kayıt sırası) sonra
  türün kendi alanları.
- `TrailParticle`: sabit bir hedefe aynı `1/kalanTik` payıyla yaklaşır; rengi `ARGB.scaleRGB` ile
  her kanal kendi `0.875+r*0.25` çarpanıyla (8 bitlik kanal üstünde kesilip [0,255]'e kırpılır,
  sonra 255'e bölünür); ışığı sabit tam parlaklık (`15728880`).

## Kapsam ve dal kararı

`particle-ticks`'e 20 satır (16 + titreşim/iz). Motor artık sağlayıcılara seviye rastgelesini
verir ve spor çiçeği sayısını tutar (eklemede sınır, ölümde ve gruba girememede düşer).

## Sapmalar (açık)

- `crimson_spore` satırı yok: hızı Gauss dağılımından (Q04, `Math.log` farkı).
- Spor çiçeği sınırı (1000) oracle'da sınanmıyor (bin parçacık gerekir).
- Kasa bağlantısının yarı saydamlığı kesik çiziliyor.
- Titreşim parçacığının blok hedefli durumu sınandı; entity hedefli (izlenen bir varlığa doğru,
  `EntityPositionSource`) durum oracle'da yok — tek parçacıklık `particle-ticks` koşumunda izlenen
  bir entity fikstürü kurmak gerekirdi. Rust tarafı entity hedefini çözüyor (`ClientWorld` üstünden
  id ile), yalnız kanıtlanmadı.
- Titreşim parçacığının çizimi (iki döndürülmüş dörtgen, `extractRotatedQuad`) bu dilimde yok;
  `photon-render`'ın `quad()` ilkeli hazır (`crates/photon-render/src/particles.rs`), yalnız
  bağlanmadı. Dünya tarafı tiki (konum, ömür, silinme, ışık) kanıtlı; render'da iz parçacığıyla
  aynı genel kamera-dönük dörtgenle çiziliyor (yanlış ama görünür).
