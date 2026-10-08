# T70e–T70h — Blokların rastgele görünüm tiki — [client-parity]

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

- Ateş (T70f): 1/24 ile `fire.ambient`. Altı yanmıyor ve üst yüzü sağlam değilse yanan her
  komşu yüzden ikişer büyük duman (`0.1F` ince şerit), değilse kendi içinden üç duman çıkar.
  Ruh ateşi her yüzü yanar sayar. Yanma `FireBlock`'un ateşleme olasılığına bakar, suya
  batmışta sıfırdır. Bu tablo oracle'dan yakalanır (`movement/fire-flammability`,
  `python3 tools/generate/fire.py <capture>/outputs.jsonl`) ve her koşuda tümüyle
  karşılaştırılır.
- Kamp ateşi yanarken 1/10 ile çıtırdar. Ruh olmayanı 1/5 ile kıvılcım atar; döngü sınırı
  (`nextInt(1)+1`) her turdan önce yeniden çekilir.
- Mum her fitilde 0.3 ile duman ve 0.17 ile ses çıkarır, her durumda küçük alev bırakır.
  Fitiller `CandleBlock.PARTICLE_OFFSETS`'ten, mumlu pastada üstte tek fitil.
- Fırın, izabe fırını ve tütsülük 0.1 ile ses çıkarır. Önlerinden duman çıkar (fırında alev
  de); tütsülükte duman üstten çıkar. Simya standı her seçilişte duman bırakır.

- T70g: nether portalı 1/100 ile `portal.ambient` çalar. Dört portal zerresinin hızı
  `(r−0.5)*0.5`'tir; eksenine göre yanlarından `±0.25` ve `r*2F*flip` ile çıkar. Spor çiçeği bir
  spor düşürür, altındaki ±10 bloklukta dolu olmayan 14 yere hava zerresi koyar. Miselyum 1/10
  ile puf çıkarır. Ağlayan obsidyen 1/5 ile rastgele bir yönde (üst hariç) komşusu örtmüyorsa
  gözyaşı damlatır. Yeniden doğma çapası doluyken ters portal zerresi bırakır, 1/100 ile ses
  çalar. Uç çubuğu 1/5 ile boyunca parıltı bırakır (`EndRodParticle`: yerçekimi 0.0125,
  sürtünme 0.91, bloklardan geçer, ikinci yarıda `0xF2DEC9`'a solar).

- T70h: yaprak bloğu kendi olasılığıyla (`leafParticleChance`, oracle'dan yakalanan tablo:
  `python3 tools/generate/leaves.py <capture>/outputs.jsonl`) altı tam üst yüzlü değilse yaprak
  düşürür. Renkli yaprak bloğun dünyadaki rengini, öbürleri kendi türünü ve rengini alır.
  `FallingLeavesParticle` 300 tik yaşar; dönüşü ±30°, ivmesi ±5° ile başlar. Rüzgârla ya akar
  (`pow(yaş, 1.25)`) ya döner (`cos/sin`). Yerde ya da duvar hızını sıfırlayınca biter.

## Kapsam ve dal kararı

Satırlar: 31×31 meşale düzlemi (dokuz tür/yön karışık); lav ve su düzlemleri; ocak düzlemi
(kamp ateşi, fırınlar, mumlar, simya standı); yerde ve tahtalar arasında asılı ateş, ruh ateşi.
Sıvı altı damlaları `animate-drips` satırındadır (T73b).

## Sapmalar (açık)

- Biyomun ortam parçacıkları ve yaratıcıdaki işaretçi parçacığı yok.
- `animate-leaves` özeti JVM `Math.sin/cos/pow` intrinsic'lerinin 1 ulp farkıyla tutmuyor (Q04
  sınıfı); yaprakların yağmur damlası yok. Öteki blokların `animateTick`'i T70i'de.
