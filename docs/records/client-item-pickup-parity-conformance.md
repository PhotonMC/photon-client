# T77 — Eşya toplama animasyonu ve `TakeItemEntity` — [client-parity]

Bu kayıt alınan bir varlığın alana doğru uçuşunu kapsar: `ItemPickupParticle`,
`ItemPickupParticleGroup.ParticleInstance.fromParticle` ve `ClientPacketListener.handleTakeItemEntity`.
Kanıt sınıfı `client-parity`'dir. `movement/item-pickup`'ta gerçek Java parçacığı her tik yer
değiştiren bir hedefle (zırh askısı) tiklenir, grubun çizim konumu her yarım tikte kameraya göre
okunur. Rust aynı girdiden bit bit aynı konumları üretir. Paket okuyucu birim testlidir.

## Java kaynak incelemesi

- Paket üç varint taşır: alınan varlık, alan, adet. Alan bir canlı değilse ya da yoksa yerel
  oyuncu alır.
- Varlık varsa ses çalar. Küre `experience_orb.pickup` 0.1 ile, perde
  `(r−r)*0.35+0.9`. Öteki `item.pickup` 0.2 ile, perde `(r−r)*1.4+2`. İkisi de dinleyici
  rastgelesinden, `PLAYERS` kategorisinde.
- Varlığın görüntüsü `extractEntity(from, 1.0F)` ile alınır: konum tikin sonundaki, yaş
  `tickCount+1`. Sonra eşyanın yığını adet kadar küçülür ve boşalırsa varlık kalkar. Küre
  kalır (sunucu kaldırır), başka varlıklar (ok, trident) kalkar.
- Parçacık `ITEM_PICKUP` grubundadır. Hedef alanın ayağı ile gözünün ortasıdır. Her tik
  `life` artar, 3'te biter. Önceki hedef saklanır, yenisi okunur.
- Çizimde `t = ((life+partial)/3)²`. Hedef yarım tikle iki hedef arasında, konum
  başlangıçla hedef arasında `t` ile ara değerlenir (`double`).

## Kapsam ve dal kararı

Üç satır var: duran, yürüyüp uzaklaşan, eğilip göz yüksekliği değişen alan. Her biri ömrünü
geçecek kadar tiklenir. Eşyalar ve küreler ölümden kopyalanan görüntüleriyle çizilir.

## Sapmalar (açık)

- Uçan varlığın gölgesi çizilmiyor; ok ve trident gibi öteki varlıklar uçarken görünmüyor.
- Işık alındığı yerdeki bloktan çizim anında okunuyor (Java'da alındığı anda dondurulur).
- Dinleyici rastgelesi parçacık paketlerininkinden ayrı (dağılım aynı).
- Kırılan eşya parçacıkları (`BreakingItemParticle`) T77b'de.
