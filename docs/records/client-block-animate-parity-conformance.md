# T70e — Blokların rastgele görünüm tiki — [client-parity]

Bu kayıt `ClientLevel.animateTick` döngüsünü ve ilk blokları kapsar: meşale ailesi, kızıltaş
meşaleleri, lav ve su. Kanıt sınıfı `client-parity`'dir. `movement/particle-engine`'de gerçek
`ClientLevel.doAnimateTick` tohumlu bir blok rastgelesiyle, blok ve sıvı düzlemlerinin olduğu
sahnede çağrılır. Seçilen bloklar, üretilen parçacıklar ve ortam dışı sesler tohumlu motorla
Rust'a bit bit eşitlenir.

## Java kaynak incelemesi

- `Minecraft.tick`: duraklatılmamışken varlıklardan sonra, parçacık motorundan önce
  `animateTick(oyuncunun bloğu)` çalışır. Tik başına 667 kez önce yarıçap 16, sonra 32 ile
  `doAnimateTick` çağrılır.
- Seçim seviye rastgelesindendir: her eksende `nextInt(r) − nextInt(r)`. Bloğun ve sıvının
  çizimi ayrı bir rastgele (`createThreadLocalInstance`) kullanır. Sıvılar için damla denemesi
  `nextInt(10) == 0` ile yine seviye rastgelesinden çekilir, sıvının alt yüzü sağlamsa
  (`isFaceSturdy(DOWN)`) yer de oradan seçilir.
- Meşale: `(x+.5, y+.7, z+.5)`'te önce duman, sonra türünün alevi (alev, ruh, bakır). Duvar
  meşalesi duvardan 0.27 dışarı, 0.22 yukarı.
- Kızıltaş meşalesi yalnız yanarken `±0.1` sapmalı `REDSTONE` tozu bırakır (duvardakinde aynı
  kaydırmayla).
- Lav: üstü hava ise 1/100 ile kıvılcım ve `lava.pop`, 1/200 ile `lava.ambient` (ortam). Su:
  akan ve düşmeyen su 1/64 ile `water.ambient`; durgun ya da düşen su 1/10 ile `underwater`.
  Her `playLocalSound` seviye rastgelesinden tohum çeker.

## Kapsam ve dal kararı

İki satır var: 31×31 meşale düzlemi (dokuz tür/yön karışık) ve lav, durgun, akan ve düşen su
düzlemleri. Sıvıların altı hava; damla yeri seçilmez ama çekilişi sınanır.

## Sapmalar (açık)

- Biyomun ortam parçacıkları ve yaratıcıdaki işaretçi parçacığı yok.
- Damla parçacığı yok (T73b); öteki blokların `animateTick`'i T70f'de.
