# P12.d — Sağ tık, blok koyma ve sınıf kuralları — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity`

## Yaklaşım: kural, sınıfın kendi kuralıdır

Referansta bir bloğun koyulunca alacağı durum, yerine başka blok konup
konamayacağı, orada durup duramayacağı, kullanılınca ne yaptığı ve komşusu
değişince nasıl şekil aldığı, sınıf hiyerarşisinde bu yöntemi tanımlayan
en yakın sınıfın kuralıdır. `block/behaviours` her blok için bu sınıfları
(`getStateForPlacement`, `canBeReplaced`, `canSurvive`, `useWithoutItem`,
`useItemOn`, `updateShape`), durum başına `canBeReplaced` bayrağını,
`instanceof StairBlock`'u ve düştüğü itemi; her item için sınıfını, `useOn`
ve `use` kurallarının sınıfını, varsayılan `Item.use`'un bir şey yapıp
yapmadığını ve koyduğu bloğu referanstan alır
(`crates/voxel-data/src/block/behaviour.rs`, `tools/generate/block_behaviour.py`,
her koşuda karşılaştırılıyor).

Rust (`crates/voxel-world/src/block/rules.rs`) kuralları sınıf başına yazar; blok,
tablodaki sınıf üzerinden kuralı bulur. Kuralı yazılmış sınıflar:
yerleştirmede `Block` (464 blok), `SlabBlock` (68), `StairBlock` (64),
`RotatedPillarBlock` (57); değiştirilebilirlikte `BlockBehaviour` ve
`SlabBlock`; `canSurvive`, `useWithoutItem` ve `useItemOn`'da varsayılan
`BlockBehaviour`; `updateShape`'te varsayılan ve `StairBlock`. Kuralı
yazılmamış bir sınıfa denk gelen kullanımda paket yine gönderilir, kol
sallanır, istemci tahmin yapmaz; sunucunun güncellemesi sonucu getirir.

## Ne pinlendi

- `Minecraft.tick` ve `handleKeybinds` sırası: `rightClickDelay` geri
  sayımı, saldırı tıklamaları, kullanım tıklamaları, basılı kullanım tuşu
  (dört tick'te bir), basılı saldırı, ıskalama beklemesi.
- `startUseItem`: kırarken kullanılmaz; her el sırayla önce hedef bloğa
  (`useItemOn`), sonra kendi başına (`useItem`) dener; başarıda kol
  sallanır ve durur, başarısızlıkta durur.
- `MultiPlayerGameMode.useItemOn` ve `useItem`: dünya sınırı, tahmin sıra
  numarası, sonuç ne olursa olsun giden paket (boş elle bir bloğa sağ tık
  iki el için iki `use_item_on` gönderir); izleyici; eğilip eşya tutarken
  bloğun kullanımının atlanması; `TRY_WITH_EMPTY_HAND` ve ana el.
- `ItemStack.useOn`: inşa edemeyen oyuncu (adventure) `PASS`.
- `BlockItem.place` ve `BlockPlaceContext`: tıklanan blok değiştirilebilirse
  onun yeri, değilse yüzün önü; `canPlace`; yerleştirme durumu; `canSurvive`
  ve oyuncunun yeri kaplaması; tahminli `setBlock` (bayrak 11) ve
  `updateNeighbourShapes` (batı, doğu, kuzey, güney, aşağı, yukarı sırası,
  güncelleme sınırı); yaratıcı olmayan oyuncuda yığının bir azalması.
- Yarım blok: alt/üst yarı tıklama noktasından, aynı yarım bloğun üstüne
  çift yarım blok, su kaynağında `waterlogged`. Merdiven: oyuncunun yönü,
  yarı, komşu merdivenlere göre köşe şekli ve koyulunca komşu merdivenin
  yeniden şekillenmesi. Kütük: tıklanan yüzün ekseni.
- Kırmadaki tahminli kırma da artık aynı `setBlock` yolundan komşu şekil
  güncellemesi yapar.
- Paketler: `use_item_on` (el, konum, yön, bloğa göre tıklama noktası,
  içeride, dünya sınırı), `use_item` (el, sıra, bakış), kol sallama (el).

## Senaryolar

`tools/behavior/cases.py` içindeki `movement/using`. Oracle
[kırma kaydındaki](block-breaking-conformance.md) gibi gerçek `LocalPlayer`
ve `MultiPlayerGameMode` üzerinde `Minecraft.startUseItem`'ı yansımayla
çağırır; oyuncu gerçek istemcideki gibi seviyenin varlıklarından biridir.
Her tick: gönderilen paketlerin baytları, izlenen blokların durumu, tahmin
sıra numarası, `rightClickDelay`, iki elin yığın sayısı ve hareket çıktısı.

Dallar: zemine taş (basılı tutma dahil), boş el, havada kullanım, alt yarım
blok ve üstüne çift, yan yüzde üst ve alt yarı, kütük ekseni, merdivenin
komşu merdivenle köşe yapması, üst yarı merdiven yönü, suya yarım blok,
oyuncunun durduğu yere koyma, creative'de sayı, çalışma masası kullanımı,
eğilerek (ilk tick'te eğilme henüz yok), sol elle koyma, adventure, son
bloğu koyma.

## Sonuç

`compare --family block --family movement`: `parity_passed`; math dışındaki
bütün aileler (907 vaka) `parity_passed`.

Canlı: vanilla 26.2 sunucusunda betikli sağ tık göl dibinde dört blok
koydu; sunucu aynı blokları aynı yerlerde doğruladı.

## Eşitlik olmayanlar ve açıklar

- Kuralı yazılmamış sınıflar (tabloda listeli): yerleştirmede 108 sınıf
  (duvar, parmaklık, kapı, meşale, bitkiler…), 11 değiştirilebilirlik, 62
  `canSurvive`, 52 `useWithoutItem`, 25 `useItemOn`, çoğu `updateShape`
  sınıfı. Bunlarda istemci tahmin yapmaz; kol sallanıp paket gider. Gerçekte
  `PASS` dönen bir blok kullanımında (demir kapı gibi) Java eşyayı koymaya
  geçer, burada geçmez.
- `BlockItem` dışındaki item sınıfları (`DoubleHighBlockItem`,
  `StandingAndWallBlockItem`, tabelalar, kovalar, aletler…) ve varsayılanı
  bir şey yapan `Item.use` (yemek, kuşanma, kalkan) yazılmadı; eşya kullanımı
  (`isUsingItem`) ve bekleme süreleri yok.
- Yer kaplama yalnız yerel oyuncuya bakar; diğer varlıklar sayılmaz.
  Adventure `can_place_on` ve `can_break` yüklemleri değerlendirilmez.
- Sol elin kol animasyonu yok; eşya kullanılınca eldeki eşyanın inip kalkması
  (`itemUsed`) yok; koyma sesi yok.
- `updateNeighborsAt` (bayrak 1) istemcide yan etkisi olan bloklar için
  çalışmaz.
