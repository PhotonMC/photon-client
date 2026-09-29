# P18.a — Eşya modelleri ve hotbar ikonları — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity` (model ve geometri), canlı gözlem (GUI pikselleri)

## Ne pinlendi

- `items/*.json` eşya modeli ağacı (`src/render/item_model.rs`): `model`,
  `composite`, `condition`, `select`, `range_dispatch`, `special`, `empty`.
  Varsayılan yığın için koşullar yanlış, `select` yalnız `display_context`
  ile seçer, `range_dispatch` değeri sıfır; `special` ve `empty` bir şey
  çizmez. Tint kaynakları: sabit, çimen iklimi, varsayılanlı olanların
  varsayılanı.
- Model `display` alanı (`src/render/model.rs`): dokuz bağlam,
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

## Senaryolar

`tools/behavior/cases.py`: `model/display-*` ve `image/item-side-faces`.
Bütün math dışı aileler (908 vaka) `parity_passed`.

Canlı: vanilla 26.2 sunucusunda hotbar'a taş, çimen, elmas kılıç, meşe
merdiven, elma, meşale, çit, cam, yay ve sol ele kütük verildi; hepsi
yerlerinde, blok eşyalar izometrik, düz eşyalar önden, sayılar doğru.

## Eşitlik olmayanlar ve açıklar

- GUI piksel çıktısı oracle ile karşılaştırılmıyor; yalnız canlı gözlem.
- Eşya dokuları referansta ayrı `items` atlasındadır; burada blok atlasına
  eklendi. GUI'de en yakın texel ve sıfırıncı seviye okunduğu için görüntü
  farkı beklenmez, ama atlas yerleşimi referansla aynı değil.
- Yığın bileşenleri okunmuyor: bileşene bağlı özellikler (iksir rengi,
  boyalı zırh, pusula, saat, dayanıklılık çubuğu, bekleme süresi örtüsü)
  varsayılan yığın gibi çizilir. `special` modeller (kalkan, sandık, kafa,
  bayrak…) çizilmez.
- Yığın alındığında zıplama animasyonu (`popTime`) yok. Ana kol hep sağ
  sayılıyor; sol el yuvası hep solda.
- Referans her eşyayı ayrı bir atlas hücresine çizip GUI'ye kopyalar; burada
  doğrudan ekrana, eşyalar için bir kez temizlenen derinlikle çiziliyor.
- Eldeki eşya (birinci şahıs) henüz yok (M2.c).
