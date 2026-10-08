# T70 — Partikül çekirdeği: `Particle`, duman ve alev — [client-parity]

Bu kayıt, bir partikülün durumunun (`Particle`, `SingleQuadParticle`) ve ilk iki
tüketicisinin (`SmokeParticle` / `BaseAshSmokeParticle`, `FlameParticle` /
`RisingParticle`) Java 26.2 ile Rust'ta aynı olmasını kapsar. Kanıt sınıfı
`client-parity`'dir: gerçek `Provider`'lar gerçek `ClientLevel`'da partikülü yapar ve
tik tik ilerletir; yaş, ömür, konum (`x`/`xo`), hız, kutu, boyut, renk, sprite sırası ve her
kısmi tikte çizim boyutu ile ışık bit bit karşılaştırılır. Partikül türü kayıt defteri de
(`particle_types`: ad, sınırlayıcıyı aşma, yalın tür) her koşuda baştan sona karşılaştırılır.
Motor, paket ve çizim sonraki görevlerdedir ([T70b](../tasks/T70b-parcacik-motoru.md),
[T70c](../tasks/T70c-parcacik-cizimi.md)).

## Java kaynak incelemesi

- `Particle`: kurucu önce ömrü çeker (`4 / (f * 0.9 + 0.1)`), hızlı kurucu üç sapma ve iki
  hız çekimini ardından yapar; `tick` yaşı artırıp ömre ulaşınca siler, yoksa yerçekimi,
  `move`, `friction` ve yerdeyken yatay 0.7 ile çarpar. `move` çarpışmayı kutuyla çözer
  (`collideBoundingBox`, dünya sınırı yok); dikey hareket 1e-5'in altına inince parçacık
  bir daha çarpışma aramaz (`stoppedByCollision`).
- `BaseAshSmokeParticle`: sürtünme 0.96, dikey hareket engellenince yatay 1.1 ile hızlanır,
  renk tek bir rastgele gri, boyut `0.75 * scale`, ömür `max(1, maxLifetime / (f*0.8+0.2) * scale)`;
  sprite yaşa göre seçilir (`index * (n-1) / max`), dörtgen ilk 1/32 ömürde büyür.
- `RisingParticle` / `FlameParticle`: konum kurucuda rastgele kayar ama kutu kaymaz (ilk tikte
  konum kutudan yeniden okunur); `move` çarpışmasızdır; dörtgen `1 - s²/2` ile küçülür; ışık
  `addSmoothBlockEmission` ile yaşla artar; sprite motorun rastgele kaynağından seçilir.
- Rastgele kaynak: her partikül `RandomSource.create()` ile kendi kaynağını saat tohumuyla yapar.
  Oracle tohumu sabitler: tohum sayacı (`RandomSupport.SEED_UNIQUIFIER`) saatin beklenen
  değerini silecek biçimde kurulur ve partikül, kaynağı istenen tohumla başlayana dek yeniden
  yapılır (saatin sapması her denemede ölçülüp ayarlanır). Kurucunun çekimleri böylece
  Rust'ta aynı tohumdan sınanır.

## Kapsam ve dal kararı

`movement/particle-ticks` 16 satır, `movement/particle-types` 1 kayıt. Dallar: kurucu çekimleri
(üç tohum), itme, sprite sayısı (1, 3, 8), çarpışma (tavan, duvar, zemin, yarım blok),
`speedUpWhenYMotionIsBlocked`, ömrün bitişi; alev için çarpışmasızlık (tavan ve taş içinde),
motor tohumuyla sprite seçimi, küçük alev. Aynı kodun yalnız sabiti değişen partikülleri
(`soul_fire_flame`, `copper_fire_flame`) tablo satırıdır, ayrı vaka değildir.

## Sapmalar (açık)

- Işık motoru: oracle sahnesi ışık motorunu çalıştırmaz (gökyüzü ışığı 15, blok ışığı 0);
  ışık motorunun kendisi `light` ailesinde kanıtlıdır.
- Konuma bağlı çarpışma şekli (`PositionCollisionContext`) taşınmadı: iskele (`scaffolding`) gibi
  bağlama duyarlı bir blok partikülün kutusuna girerse partikül o tik yerinde kalır.
- Tür kaydı elle yazılmıştır (`photon-data/src/particle_type.rs`); sürüm değişince oracle'ın
  `particle_types` çıktısından yenilenir.
