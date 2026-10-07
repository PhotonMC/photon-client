# T71 — Blok parçacıkları: `TerrainParticle`, kırılma ve çatlama — [client-parity]

Bu kayıt, bir bloğun kırıntısının (`TerrainParticle`), sağlayıcılarının (`block`, `block_crumble`,
`dust_pillar`) ve kırılan ya da kırılmakta olan bloğun kırıntılarının (`ClientLevel.addDestroyBlockEffect`,
`addBreakingBlockEffect`) Java 26.2 ile Rust'ta aynı olmasını kapsar. Kanıt sınıfı `client-parity`'dir:
gerçek sağlayıcılar tohumlu partikülü yapar ve tik tik ilerletir; yaş, ömür, konum, hız, kutu, boyut,
renk (sabit tintler dahil), dokunun gösterdiği çeyrek (`getU0`..`getV1`) ve ışık bit bit
karşılaştırılır; kırılma etkisinin her kırıntısının yeri (ve çatlamada boyutu) karşılaştırılır.
`BlockParticleOption` paketi (`block` türleri) çözülür; `LevelEvent 2001` dünyada bekletilir. Çizim,
doku ve tint çözümü, oyun moduna bağlama [T71b](../tasks/T71b-blok-parcacigi-cizimi.md)'dedir.

## Java kaynak incelemesi

- `TerrainParticle`: `SingleQuadParticle` kurucusunun çekimlerinden sonra yerçekimi 1, renk 0.6'nın
  tint kaynağının `colorAsTerrainParticle`ı ile çarpımı, boyut yarıya, ardından iki çekim (`uo`, `vo`,
  `[0, 3)`); doku, bloğun parçacık malzemesinin dörtte biridir: `getU0 = getU((uo+1)/4)`, `getU1 = getU(uo/4)`,
  `getV0 = getV(vo/4)`, `getV1 = getV((vo+1)/4)`.
- Sağlayıcı: hava, hareketli piston ve `noTerrainParticles` (bariyer, yapı boşluğu) kırıntı yapmaz.
  `block_crumble` hızı sıfırlar ve ömrü `nextInt(10) + 1` yapar (motorun rastgelesi); `dust_pillar` hızı
  `gauss/30, yAux + gauss/2, gauss/30` ve ömrü `nextInt(20) + 20` yapar.
- `addDestroyBlockEffect`: şeklin her kutusunda, kutunun genişliğinin 0.25'e bölümü (en az 2) kadar
  hücre; her hücreden bir kırıntı, hız hücrenin kutudaki yerinden 0.5 çıkarılmış hâli. `addBreakingBlockEffect`:
  seviyenin rastgelesinden kutunun içinde bir yer, bakılan yüze yapışık; `setPower(0.2)` ve `scale(0.6)`.
- `ParticleTypes` alan türü `BlockParticleOption` olanlar blok durumunu (VarInt) paketin sonunda taşır.

## Kapsam ve dal kararı

`movement/particle-terrain` 28 satır: sağlayıcılar (taş, iki tohum, tintli iki yaprak, tintsiz çimen bloğu,
dört türde kırıntı yapılmayan durum, ufalanma, toz sütunu), kırılma (tam blok, plaka, merdiven, iç köşe
merdiven, çit, sandık, meşale, bariyer, hava; çok kutulu şekiller ayrı dal) ve çatlama (altı yön, plaka
iki yan, bariyer). Kırılmada her kırıntının kendi rastgele kaynağı bilinmez (Java onu kurucuda, saatle
tohumlar), bu yüzden yer ve sayı karşılaştırılır; kırıntının içi sağlayıcı satırlarıyla sınanır.

## Sapmalar (açık)

- Biyoma bağlı tintler (çim, yaprak, su) oracle'da yok (başsız oracle'da renk haritaları boş); Rust
  tarafında bunlar [T71b](../tasks/T71b-blok-parcacigi-cizimi.md)'de gerçek tint çözümüyle gelir. Oracle
  adaptörü yalnız sabit tintleri (ladin, huş) bilir.
- `block_marker` ve `falling_dust` sağlayıcıları ayrı sınıflardır, taşınmadı.
