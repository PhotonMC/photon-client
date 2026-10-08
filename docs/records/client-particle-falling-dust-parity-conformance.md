# T74b — Düşen toz ve düşen blok animasyonu — [client-parity]

Bu kayıt `FallingDustParticle`'ı, `FallingBlock.animateTick`'i ve kumun çöl sesi çekilişini kapsar.
Kanıt sınıfı `client-parity`'dir. `movement/particle-engine`'de gerçek sağlayıcı (doğrudan istenen
toz) ve `doAnimateTick` (boşluk üstündeki düşen bloklar) koşar. Blokların harita rengi ve düşen
blokların toz rengi tabloları `movement/block-map-colors` ile her koşuda tümüyle karşılaştırılır.

## Java kaynak incelemesi

- Sağlayıcı hava dışında görünmez çizilen bloklardan toz çıkarmaz (bariyer, ışık, sıvılar,
  kabarcık sütunu, end portalı/geçidi, yapı boşluğu).
- Renk sırası: düşen bloğun `getDustColor`'ı (kum ve çakıl kendi rengi, örs ve beton tozu harita
  rengi, ejderha yumurtası siyah). Değilse kırıntının alacağı renk kaynağı
  (`colorAsTerrainParticle`), o da yoksa durumun harita rengi (`MapColor.col`).
- Parçacık: boy ×0.675, ömür `int(max(int(32/(r*0.8+0.2))*0.9F, 1))`. Dönüş hızı
  `(r−0.5)*0.1`, ilk dönüş `r*2π`. Her tik dönüş `π*hız*2` artar, yerde sıfırlanır. Hareketten
  sonra `yd −= 0.003F`, en fazla `−0.14F`. Sürtünme yoktur, boy ilk 1/32 ömürde büyür.
- `FallingBlock.animateTick`: 1/16 ile, altı boşsa (`isFree`: hava, ateş etiketi, sıvı ya da
  yerine konabilir) bloğun altından (`y−0.05`) toz düşer. Kum ve kızıl kum ayrıca, üstü hava ise
  `nextInt(2100)` çeker. Dört yöndeki sekiz blok ötesi sütunların dördünde gökyüzü altında kum
  varsa (`WORLD_SURFACE` yüksekliği) ortam sesi çalar.
- Görünmez bir blok için sağlayıcı parçacık tohumu harcamaz; Rust da tohumu yalnız parçacık
  yapılırken çeker.

## Kapsam ve dal kararı

Satırlar: yedi düşen blok türünden 31×31 tavan ve doğrudan istenen sekiz toz (taş, çim, ladin
yaprağı, bariyer, su, hava, beton tozu, kırık örs). Harita rengi tablosu oracle'dan yakalanır:
`python3 tools/generate/map_colors.py <capture>/outputs.jsonl`.

## Sapmalar (açık)

- Havai fişek parçacıkları T74c'de.
