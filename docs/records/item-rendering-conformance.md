# P18.a — Eşya modelleri, hotbar ikonları ve eldeki eşya — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity` (model ve geometri), canlı gözlem (GUI pikselleri)

## Ne pinlendi

- `items/*.json` eşya modeli ağacı (`crates/voxel-render/src/item_model.rs`): `model`,
  `composite`, `condition`, `select`, `range_dispatch`, `special`, `empty`.
  Varsayılan yığın için koşullar yanlış, `select` yalnız `display_context`
  ile seçer, `range_dispatch` değeri sıfır; `special` ve `empty` bir şey
  çizmez. Tint kaynakları: sabit, çimen iklimi, varsayılanlı olanların
  varsayılanı.
- Model `display` alanı (`crates/voxel-render/src/model.rs`): dokuz bağlam,
  `ItemTransform` sınırları (öteleme on altıda bir ve ±5, ölçek ±4), sol
  elin sağ eli miras alması. `block/model` ailesindeki
  `display-right-hands` ve `display-clamped-and-left` vakaları.
- `builtin/generated` ve `ItemModelGenerator`: katman başına ön ve arka
  yüz (z 7.5–8.5), opak/saydam piksel sınırlarındaki yan yüzler ve 0.1'lik
  UV içe çekme. Yan yüz listesi `image/item-side-faces` vakasında on iki
  eşya dokusu için referansın `getSideFaces`'iyle karşılaştırılıyor.
- `gui_light`: `side` modeller `ITEMS_3D`, `front` modeller `ITEMS_FLAT`
  ışığıyla (`Lighting` pozları, `minecraft_mix_light`).
- Hotbar (`Hud.extractItemHotbar`, `extractSlot`, `itemDecorations`):
  dokuz yuva `(merkez - 90 + i·20 + 2, yükseklik - 19)`, sol el yuvası ve
  `hud/hotbar_offhand_left`, sayı bir değilse `x + 17 - genişlik`,
  `y + 9`'da gölgeli beyaz metin (gölge rengin dörtte biri, bir birim sağ
  alt).
- Yuvaya çizim (`GuiItemAtlas.drawToSlot`): yuva merkezi, yuva boyu kadar
  blok ve ters y, ardından GUI bağlamındaki dönüşüm (yoksa yarım blok geri);
  derinlik testli, 0.1 altı alfa kesilen ayrı bir Vulkan hattı.
- Eldeki eşya (`crates/voxel-game/src/item_in_hand.rs`, `held_item_vertices`):
  `ItemInHandRenderer.tick` sırası (tuşlardan sonra, oyuncu tick'inden
  önce), gösterilen yığının envanterden geri kalması, yığın değişince elin
  0.4'lük adımlarla inip onda birin altında yeni yığına geçmesi, yığınların
  nesne kimliğiyle karşılaştırılması (yerinde azalan yığın aynı nesnedir),
  `hand_animation_on_swap` ve `swap_animation_scale`, `itemUsed` (kullanım
  yığını azalttığında ya da oyuncu yaratıcıdayken el dibe iner). Boş ana el
  kolu, dolu el eşyayı çizer; sol el eşyası solda. Poz: `bobView`, elin
  bakışın gerisinde kalması, `applyItemArmTransform`, varsayılan `WHACK`
  sallanması (`swingArm`, `applyItemArmAttackTransform`), birinci şahıs
  `ItemTransform.apply` (sol elde öteleme x'i, y ve z dönüşleri ters) ve
  `rotationXYZ` (JOML portuna eklendi).

## Senaryolar

`tools/behavior/cases.py`: `model/display-*` ve `image/item-side-faces`.
Bütün math dışı aileler (908 vaka) `parity_passed`.

Canlı: vanilla 26.2 sunucusunda hotbar'a taş, çimen, elmas kılıç, meşe
merdiven, elma, meşale, çit, cam, yay ve sol ele kütük verildi; hepsi
yerlerinde, blok eşyalar izometrik, düz eşyalar önden, sayılar doğru.
Sağ elde taş, çimen bloğu ve elmas kılıç, sol elde elma ve meşale
referanstaki yerlerinde görüldü. Eldeki eşya durumunun inip kalkması iki
birim testiyle sabitlendi.

## Eşitlik olmayanlar ve açıklar

- GUI piksel çıktısı oracle ile karşılaştırılmıyor; yalnız canlı gözlem.
- Eşya dokuları referansta ayrı `items` atlasındadır; burada blok atlasına
  eklendi. GUI'de en yakın texel ve sıfırıncı seviye okunduğu için görüntü
  farkı beklenmez, ama atlas yerleşimi referansla aynı değil.
- Yığın bileşenlerinden yalnız renk kaynakları okunur (`crates/voxel-game/src/item_values.rs`):
  `dye` (`dyed_color`), `potion` (`potion_contents`: özel renk ya da
  iksirin ve özel efektlerin renklerinin genlik ağırlıklı ortalaması; iksir
  ve efekt tablosu oracle'dan), `firework` (`firework_explosion` renkleri) ve
  `map_color`. Bileşene bağlı diğer özellikler (pusula, saat, dayanıklılık
  çubuğu, bekleme süresi örtüsü, zırh süsü) varsayılan yığın gibi çizilir.
  Eşyanın varsayılan bileşenleri referansın yamasından okunur. varsayılan yığın gibi çizilir. `special` modeller `ModelPart`
geometrisiyle çizilir (`crates/voxel-render/src/special_models.rs`, ölçüler ve dokular
referansın `LayerDefinition`larından): sandıklar (kapak kapalı, çift
sandık yok), shulker kutuları, bayraklar (yalnız taban deseni ve boya
rengi; desen katmanları yok), kafalar (iskelet, wither iskeleti, zombi,
creeper, piglin, ejderha, oyuncu — varsayılan `steve` derisiyle), kalkan
(desensiz), conduit, süslü çömlek (süssüz) ve bakır golem heykelleri (ayakta
duruş). Pozlar hareketsiz başlangıç konumundadır (kafa ileri bakar, kapaklar
kapalı). `local_time` seçimi varsayılan dokuyu alır: 24-26 Aralık'ın Noel
sandık dokusu yok. Trident'in `special` modeli ve heykellerin diğer
duruşları çizilmez; boyalı kalkan, desenli bayrak ve süslü çömlek süsleri
bileşen değerleri okunmadığı için yok.
- Modelin `transformation` alanı (`model`, `special`, `composite`, `select`,
  `condition`, `range_dispatch` düğümlerinde) düğümden yapraklara birleştirilir
  ve katmanın geometrisine görüntü dönüşümünden önce uygulanır; yatağın iki
  yarısı (ayak yarısı bir blok öteye) böyle çizilir.
- Yığın alındığında zıplama animasyonu (`popTime`) yok. Ana kol hep sağ
  sayılıyor; sol el yuvası hep solda.
- Referans her eşyayı ayrı bir atlas hücresine çizip GUI'ye kopyalar; burada
  doğrudan ekrana, eşyalar için bir kez temizlenen derinlikle çiziliyor.
- Eldeki eşya pozu ve kimlik mantığı oracle ile karşılaştırılmıyor;
  referans sırası kaynaktan birebir taşındı, sonuç canlı gözlemle görüldü.
- Kullanım animasyonları (yeme, içme, yay germe, kalkan, mızrak, fırça,
  trident), harita, arbalet ve dönerek saldırı yok; mızrağın `STAB`
  sallanması `WHACK` gibi çiziliyor. Hangi elin sallandığı izlenmiyor,
  sallanma hep ana eldedir.
- Eldeki eşya blok atlasından arazinin örnekleyicisiyle, cutout olarak,
  arka yüzler atılarak çiziliyor; saydam eşyalar (renkli cam) karışmıyor.
  Işık, kolun ışığıyla aynı: seviyenin iki ışığı dönmüş normal üzerinde.
- `damage` bileşeni takas karşılaştırmasında dışlanmıyor (yama bayt bayt
  karşılaştırılıyor); hasar alan bir alet eli indirebilir. Son blok
  koyulunca yuva boşaldığından el iner, referansta inmez.
