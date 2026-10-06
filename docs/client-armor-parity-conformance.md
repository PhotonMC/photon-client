# T41b — Zırh katmanı (insansılar) — [client-parity]

Bu kayıt, oyuncu, zombi, iskelet, piglin ve aileleri (yetişkin ve bebek) üzerindeki
zırhın çizimini kapsar: hangi eşyanın hangi ekipman varlığıyla giyildiği, varlığın
katmanları, dokuları ve boya rengi, ve zırh modellerinin giyenin pozuyla çizilmesi.
Kanıt sınıfı `client-parity`'dir: bütün ekipman varlıkları (`equipment/*.json`) gerçek
`EquipmentClientInfo.CODEC` ile okunup her insansı katman türü için katman, doku yolu ve
`EquipmentLayerRenderer.getColorForLayer` rengi (boyasız ve boyalı) Rust ile bit bit
karşılaştırılır; zırh eşyasının hangi varlıkla giyildiği (`Equippable.assetId`) eşya
tablosunda her koşuda karşılaştırılır. Zırh modelleri model katman tablosundan
(`<aile>#helmet|chestplate|leggings|boots`) pişirilir; poz, giyenin poz kodudur
(`entity/*-model` ailelerinde zaten karşılaştırılır). Canlı gözlem (zırhlı oyuncu, zombi,
iskelet) `server-conformance`.

## Java kaynak incelemesi

- `HumanoidArmorLayer`: sırayla göğüs, bacak, ayak, baş; parça `equippable` bileşeninin
  varlığı varsa ve yuvası uyuyorsa çizilir (`shouldRender`); katman türü bebekte
  `humanoid_baby`, bacakta `humanoid_leggings`, ötekilerde `humanoid`; bebek ayrı model
  takımı kullanır. Zırh modelleri giyenin model sınıfıyla (`ZombieModel` vb.) kurulur: poz
  aynıdır.
- `EquipmentLayerRenderer.renderLayers`: varlığın katmanları sırayla; her katmanın rengi
  `getColorForLayer` (boyanabilir katman boyasız `colorWhenUndyed`, boyalıysa boya; boyası
  ve rengi yoksa katman çizilmez; boyanamaz katman beyaz). Doku
  `textures/entity/equipment/<tür>/<ad>.png`. Parıltı (`armorEntityGlint`) ve süs
  (`ArmorTrim`) ayrı geçişlerdir.
- `DyedItemColor.getOrDefault`: boya `ARGB.opaque(rgb)`, yoksa sıfır.

## Veri

`Equippable.assetId` eşya tablosuna girdi (`Equip.asset`, `tools/generate/items.py`,
oracle `block/item-declarations`). Ekipman varlıkları jar'dan çalışma anında okunur
(`voxel-render/equipment.rs`), dokuları giyilebilir varlıkların hepsi için varlık
başlangıcında entity dokusuna paketlenir.

## Kapsam

`entity/equipment-layers` 39 satır (her ekipman varlığı × üç katman türü; boyasız ve
boyalı renk). Aile içi zırh türleri veri satırıdır.

Süs (T41e): `image/trim-sprite-id` 246 satır gerçek `ArmorTrim.layerAssetId`'e karşı (desen × malzeme × zırh varlığı: `iron_darker` gibi geçersiz kılmalar, düz ve pantolon katmanı) ve `image/palette-permutation` 3 satır gerçek `PalettedPermutations` eşlemesine karşı (sentetik resim ve palet: anahtar renk değişir, alfa çarpılır, diğer opak renkler kalır). Oyunda süs atlası açılışta üretilir (`Armor::trim_images`, `voxel-render::trim`), `trim` bileşeninin kayıt numaraları sunucunun adlarıyla çözülür, süs zırh katmanının üstüne boyasız çizilir (yavrularda yok). Canlı: süslü zombi görüldü. Süs kayıt verisi vanilla veri paketinden; sunucunun özel desenleri okunmaz.

Elytra (T41f): `entity/elytra-model` 8 satır gerçek `ElytraModel.setupAnim`'e karşı (büyük ve yavru, dinlenme/çömelme/süzülme açıklıkları) ve `movement/elytra-state` 13 senaryo gerçek `ElytraAnimationState`'e karşı (gerçek uzaktaki oyuncu: durma, çömelme, düz süzülme, dik ve sığ dalış, yükselme, süzülmenin bitmesi, çömelerek süzülme; kanat açıklığı üç kısmi tikte bit bit). Oyunda `ClientEntity::elytra_rotations` her yaşayan varlığın tikinde (yerel oyuncunun kendi durumu da) ilerler, `Armor::wings` göğüsteki eşyanın ekipman varlığının `wings` katmanını `elytra#main` modelinde, arkaya 1/8 blok kaydırarak uzaktaki oyuncuda, yerel oyuncunun üçüncü şahıs modelinde ve insansı mobda çizer. Canlı: elytralı zombinin arkasında kanatlar görüldü.

## Sapmalar (açık)

- Büyü parıltısı (`armorEntityGlint`) çizilmiyor ve eşya yamasındaki `equippable` okunmuyor ([T41e2](tasks/T41e2-zirh-parilti-equippable.md)). Dekal desenli süs (`decal: true`; vanilla'da yok) süs katmanı gibi çizilir, derinlik eşitliği yok.
- Boya yamadan okunur.
- Oyuncu dokusu/pelerin dokusu (PLAN-M9), süzülen oyuncunun gövde eğimi ve kafatası modelleri ([T41f3](tasks/T41f3-kafatasi-durbun-suzulme.md)); yerel oyuncu süzülmez; pelerin T41c'de; at/kurt/lama gövde zırhı,
  eyer katmanları ilgili aile görevlerinde. Zırh askısı T40'ta.
- Oyuncu zırhı için `wearing` yamaları (gizli ayrıntılar: `hide_tooltip`) etkisiz.
