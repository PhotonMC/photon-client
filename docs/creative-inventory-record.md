# P22.d.2 — Yaratıcı envanter ekranı — oracle tablosu ve canlı oturum kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** sekme tablosu `client-parity` (oracle, her koşuda
karşılaştırılır); ekranın kendisi canlı gözlem

## Sekme tablosu

`block/creative-tabs` (BlockOracle) referansın
`CreativeModeTabs.tryRebuildTabContents` çağrısını varsayılan özellik
bayrakları, operatör izni olmayan oyuncu ve sunucunun kayıtlarıyla yapar ve
her sekmeyi yazar: adı, türü, satırı ve sütunu, sağa yaslı olup olmadığı,
başlığı gösterip göstermediği, kaydırılıp kaydırılamadığı, arka plan dokusu,
başlık çeviri anahtarı, simgesi ve eşyaları. Bir eşya, ağın kodladığı haliyle
(sayı, eşya numarası, bileşen yaması) ve yamadaki bileşenlerin numaralarıyla
yazılır; 14 sekmede 1896 ayrı eşya var.

`tools/generate/creative_tabs.py` capture'dan `src/creative_tabs_table.rs`
üretir (eşyalar bir kez, sekmeler indeksle). Tablo commit'lenir; adaptörün
`creative_tabs` işlemi tabloyu aynı JSON'a çevirir ve her parity koşusunda
referansla karşılaştırılır (`block` ailesi `parity_passed`).

## Ekran

`src/creative_menu.rs` `ItemPickerMenu`u ve ekranın `slotClicked`ini taşır,
`src/render/creative_screen.rs` yerleşimi, fareyi ve çizimi:

- Pencere 5×9 yuva; satır satır kayar (`scrollTo`, `getRowIndexForScroll`,
  `subtractInputFromScroll`); kaydırma çubuğu sürüklenir, tekerlek satır
  kaydırır. Sekme düğmesi bırakılınca seçilir; son seçilen sekme ekran
  kapanıp açılınca korunur (referansta statik).
- Sekmeler: 5 üst kategori, kayıtlı hotbar'lar, arama, 5 alt kategori,
  envanter. Operatör sekmesi görünmez (`shouldDisplay` yanlış).
- Kategori ve arama sekmesinde bir yuvaya tıklamak eşya yapar, taşımaz:
  sol tık taşınana bir ekler, shift tam yığın, sağ tık bir azaltır, orta tık
  tam yığın alır, sayı tuşu hotbar yuvasını tam yığınla doldurur, Q bir ya
  da tam yığın atar. Hotbar yuvalarında sıradan `doClick` çalışır; shift ile
  tık yığını siler. Pencere dışında imleçteki yığın atılır.
- Envanter sekmesi oyuncunun kendi menüsünü referansın yerleşiminde gösterir
  (zırh, sol el, ana envanter, hotbar) ve çöp kutusu yuvası vardır: imleçteki
  yığını siler, shift ile tıklamak 46 yuvanın hepsini boşaltır.
- Paketler: hotbar/envanter değişiklikleri `SetCreativeModeSlot` ile yuva
  başına (`broadcastChanges` sırasıyla), atılan yığın yuva −1 ile ve atma
  sınırlayıcısı (`TickThrottler(20, 1280)`) sayılır. Container click paketi
  gitmez. Bileşen yaması istemci biçiminde, her değer uzunluğuyla yazılır
  ([bileşen kaydı](item-components-conformance.md)).
- Oyun modu değişince iki envanter ekranı birbirine geçer; E yaratıcıda da
  ekranı açar.

## Canlı sonuç

Vanilla 26.2 sunucusunda yaratıcı modda betikle (`tab/N`, `scroll/YÜZDE`,
`click/YUVA/DÜĞME/TÜR` ekran açıkken yaratıcı ekrana gider):

- Yığın alınıp hotbar'a konunca, sayı tuşuyla doldurulunca ve shift tıkla
  tam yığın alınınca sunucu envanteri aynı gördü (`data get entity Miner
  Inventory`); pencere dışına tıklamak yığını yere düşürdü ve kol salladı
  (yerde `minecraft:oak_stairs` durdu).
- Ekran görüntülerinde kategori, arama, kayıtlı hotbar ve envanter
  sekmeleri referansın yerleşiminde: sekme düğmeleri ve simgeleri, başlık,
  kaydırma çubuğu ve hotbar/zırh/çöp yuvaları.

## Açıklar

- `#etiket` araması ve arama sekmesinin etiket satırları yok; ad araması ve
  eşya ipuçları var ([kayıt](tooltip-record.md)).
- Kayıtlı hotbar'lar okunmaz (`hotbar.nbt`): hepsi boş, yer tutucu kâğıtlar
  gösterilir; C/X ile kaydetme ve yükleme yok.
- Envanter sekmesindeki oyuncu modeli (20 birim/blok, fareye bakar) zırhsız ve
  elsiz çizilir; etkiler paneli yok.
- `broadcastChanges` son yayından beri değişeni gönderir; burada tıklamadan
  önce ile sonra karşılaştırılır. Sunucunun tıklamalar arasında yaptığı bir
  değişiklik referansta ek bir paket üretir.
- Sunucunun özellik bayrakları okunmaz (`isFeatureEnabled`): varsayılan
  bayraklar varsayılır.
