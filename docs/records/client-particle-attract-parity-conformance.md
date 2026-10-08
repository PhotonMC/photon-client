# T75 — Portal, çekim ve asılı parçacıklar — [client-parity]

Bu kayıt bir yere çekilen ve havada asılı kalan parçacıkları kapsar: `PortalParticle`,
`ReversePortalParticle`, `FlyTowardsPositionParticle` (`enchant`, `nautilus`, `vault_connection`),
`FlyStraightTowardsParticle` (`ominous_spawning`), `SuspendedParticle` (`underwater`,
`warped_spore`, `crimson_spore`, `spore_blossom_air`) ve `SuspendedTownParticle` (`mycelium`,
`happy_villager`, `egg_crack`, `composter`, `dolphin`). Kanıt sınıfı `client-parity`'dir: gerçek
sağlayıcılar tohumlanmış parçacık, motor ve seviye rastgeleleriyle `movement/particle-ticks`'te
koşturulur, alanlar bit bit karşılaştırılır.

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

## Kapsam ve dal kararı

`particle-ticks`'e 16 satır. Motor artık sağlayıcılara seviye rastgelesini verir ve spor çiçeği
sayısını tutar (eklemede sınır, ölümde ve gruba girememede düşer).

## Sapmalar (açık)

- `crimson_spore` satırı yok: hızı Gauss dağılımından (Q04, `Math.log` farkı).
- Spor çiçeği sınırı (1000) oracle'da sınanmıyor (bin parçacık gerekir).
- Kasa bağlantısının yarı saydamlığı kesik çiziliyor.
