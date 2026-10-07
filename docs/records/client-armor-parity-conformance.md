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

Büyü parıltısı (T41e2): `entity/item-foil` 8 satır gerçek `ItemStack.hasFoil`'e karşı (her eşyanın varsayılanı, ayrıca bileşen yamasıyla büyü, parıltı geçersiz kılması açık/kapalı, geçersiz kılmanın kaldırılması, boş büyü; yama telin kendi biçiminde, Java yamayı gerçek codec'le çözer) ve `entity/glint-layers` 9 satır gerçek `RenderTypes.armorEntityGlint`'e karşı (boru hattı: `glint`, ışık ve blend `GLINT`, derinlik `EQUAL` ve yazmaz, çift yüz, doku `misc/enchanted_glint_armor`; doku matrisi `setupGlintTexturing` zaman ve parıltı hızı verilerek 16 sayı bit bit). `hasFoil`: parıltı geçersiz kılması varsa o, yoksa büyü bileşeni boş değil. Oyunda giyilen parçanın ilk çizilen katmanı (`renderFoil`) ikinci kez, doku koordinatı katmanın 0–1 uzayından glint matrisiyle (`ötele(−o0, o1)`, `Z(π/18)`, `0,16` ölçek) glint dokusuna taşınarak ve doku tekrar ederek çizilir (`Armor::glint`, `PipelineKind::Glint`: ışıksız, sis ve parıltı gücüyle sönen, rengiyle eklenen); zırh ve elytra. Canlı: büyülü ve parıltı geçersiz kılınmış zırh askılarında parıltı görüldü.

`equippable` yaması (T41e3): `entity/item-equippable` 8 satır gerçek `ItemStack.get(EQUIPPABLE)`'e karşı (eşyanın kendi bileşeni, yamanın başka varlık vermesi, giyilmeyen eşyaya yuva ve varlık, varlıksız, ses bütün olarak yazılı (menzilli ve menzilsiz), yamanın bileşeni kaldırması; yama telin biçiminde, Java gerçek codec'le çözer). `Equippable.STREAM_CODEC`'ten katmanların okuduğu iki alan (yuva ve varlık kimliği) okunur; önce ekipman sesi (kayıt numarası artı bir, ya da sıfır ve olayın kendisi) atlanır. `equippable_of`: yama ekler ya da kaldırırsa yama, yoksa eşya tablosu; `Armor::asset_for` zırh, elytra ve eyer katmanları için bunu kullanır. Canlı: `equippable` bileşenli bir çubuk zırh askısında altın göğüslük olarak görüldü.

## Sapmalar (açık)

- Parıltı gücü seçeneği varsayılan (0,75) ve parıltı hızı varsayılan (0,5) sabittir (seçenek ekranı bağlı değil); parıltı derinlik eşitliği yerine büyük-eşit testi kullanır (aynı üçgenler modelin en yakın olduğu yerde geçer); eyer ve öteki `simple_layer` katmanlarında, kafa eşyasında ve elde taşınan eşyada parıltı yok.
- Dekal desenli süs (`decal: true`; vanilla'da yok) süs katmanı gibi çizilir, derinlik eşitliği yok.
- Boya yamadan okunur.
- Oyuncu dokusu/pelerin dokusu (PLAN-M9), kafatası modelleri ([T41f4](../tasks/T41f4-kafatasi-durbun.md)); yerel oyuncu süzülmez; pelerin T41c'de; at/kurt/lama gövde zırhı,
  eyer katmanları ilgili aile görevlerinde. Zırh askısı T40'ta.
- Oyuncu zırhı için `wearing` yamaları (gizli ayrıntılar: `hide_tooltip`) etkisiz.
