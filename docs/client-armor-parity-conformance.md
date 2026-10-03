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

## Sapmalar (açık)

- Zırh süsü (`ArmorTrim`, süs atlası) ve büyü parıltısı çizilmiyor (T41e).
- Eşya yamasındaki `equippable` bileşeni (özel varlık kimliği) okunmuyor; yalnız eşyanın
  varsayılan bileşeni. Boya yamadan okunur.
- Kafadaki eşya (kafatası, balkabağı), elytra, pelerin T41c'de; at/kurt/lama gövde zırhı,
  eyer katmanları ilgili aile görevlerinde. Zırh askısı T40'ta.
- Oyuncu zırhı için `wearing` yamaları (gizli ayrıntılar: `hide_tooltip`) etkisiz.
