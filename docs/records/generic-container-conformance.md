# Sunucunun açtığı genel kaplar (sandık, fıçı, ender sandığı, dağıtıcı, hopper, shulker kutusu, çalışma tezgâhı)

Kapsam: `OpenScreen` ile açılan `generic_9x1..9x6`, `generic_3x3`, `hopper` ve
`shulker_box` menüleri. Fırın, tezgâh, örs gibi diğer menü türleri henüz
modellenmedi; sunucu bunlardan birini açarsa istemci kapanış paketini hemen
gönderir (sunucu görünmeyen bir pencereyi beklemesin).

## Menü modeli

`photon_game::container_menu::ContainerMenu` artık oyuncunun kendi menüsü ile
genel kap menülerini aynı `doClick` portuyla çalıştırır (`MenuKind`). Kap
menüsünde önce kabın kendi yuvaları, sonra ana envanterin 27'si ve hotbar'ın 9'u
gelir. Java'dan taşınanlar: `ChestMenu`/`HopperMenu`/`ShulkerBoxMenu`/
`DispenserMenu.quickMoveStack` (yalnız dağıtıcı, hiçbir şey taşınmazsa vazgeçer),
`ShulkerBoxSlot.mayPlace` (shulker kutusu shulker kutusuna girmez), sunucunun
verdiği kap kimliğiyle tıklama paketi. Kap menüsünde off hand bir yuva olmadığı
için off hand ile değiştirme tahmin edilmez; sunucunun düzeltmesi gelir.

Kabın son 36 yuvası oyuncunun envanteridir: sunucunun içerik ve yuva paketleri
envantere yazılır, sunucunun envanteri değiştirmesi kabın menüsünde görünür.

## Kanıt

- `movement/container-clicks` oracle'ına 15 satır eklendi (sandık 1/3/6 satır,
  hopper, dağıtıcı, shulker kutusu: alma/koyma, hızlı taşıma, sayı tuşları,
  atma, sürükleme, çift tıklama, klon). Java menüyü `MenuType.create` ile kurar;
  tahmin, yuvalar ve gönderilen paket baytı Rust ile birebir (`client-parity`).
- Canlı (`server-conformance`): üç eşyalı bir sandık açıldı, hızlı taşıma ve
  alma/koyma tıklandı, kapatıldı; sunucudaki sandık ve oyuncu envanteri istemcinin
  tahminiyle aynı çıktı.

## Çalışma tezgâhı

`MenuKind::Crafting` (menü türü 12): sonuç 0, 3×3 ızgara 1–9, sonra oyuncunun
27+9 yuvası. `CraftingMenu.quickMoveStack` ve sonuç yuvasının ızgarayı
tüketmesi (istemci tarafında `defaultCraftingReminder`) oracle'da 9 satırla
Java ile birebir. Tarif kitabı için aşağıdaki bölüme bakın.
Canlı: bir kütük ızgaraya taşındı, sunucunun gönderdiği sonuç hızlı taşınarak
8 tahta alındı.

## Fırınlar (fırın, yüksek fırın, dumanlayıcı)

`MenuKind::Furnace` (menü türleri 14, 10, 22): malzeme 0, yakıt 1, sonuç 2.
`AbstractFurnaceMenu.quickMoveStack`, `FurnaceFuelSlot` (yalnız yakıt ve kova,
kova 1'lik), `FurnaceResultSlot` (yalnız alınır) portlandı. İstemci ne yandığını
ve neyin pişeceğini Java gibi ayırır: yakıt süreleri oracle'dan üretilen eşya
tanımındaki `burn_time` (`FuelValues.vanillaBurnTimes`, 280 eşya), pişen eşyalar
sunucunun `UpdateRecipes` paketindeki `furnace_input`/`blast_furnace_input`/
`smoker_input` kümeleri. `ContainerSetData` (yanma ve pişme ilerlemesi) işlenir
ve ekran alev ile oku çizer. Oracle'da 8 satır Java ile birebir (Java menüyü
`MenuType.create` ile, kümeleri `ClientRecipeContainer` ile kurar). Canlı:
ham demir ve kömür hızlı taşınınca doğru yuvalara gitti, pişirme ilerledi.

## Dönüşüm menüleri (taş kesici, grindstone, cartography, loom, smithing, örs, büyü masası)

Hepsi `MenuKind` ile aynı `doClick` portundan geçer (`container_menu/transform.rs`:
yerleştirme kuralları, alma kancası, `quickMoveStack`, girdi değişince sonucu
temizleme). Grindstone ve örs dışında sonuç yuvalarını sunucu üretir; istemci girdi değişince
sonucu boşaltır ve sunucunun `SetSlot`'unu bekler (Java'nın tahmin ettiği
sonuçlar için bilinen sapma, sunucu düzeltir). Taş kesici tarifleri
`UpdateRecipes`'ten (`SlotDisplay`), loom desenleri `banner_pattern`
kayıt defteri ve etiketinden, büyü adları 62 kelimelik listeden ve
`data[3]` tohumundan gelir. `ContainerButtonClick` (17): taş kesici tarifi, loom
deseni ve büyü satırı; örste ad `RenameItem` (48) ile gider. Oracle'da her
menü için satırlar Java ile birebir; ekranlar canlı doğrulandı (taş kesici,
loom, büyü masası, örs).

Denetimle düzeltilenler: örs ad kutusu boş slotta yazı almaz ve özgün ada dönünce
`Rename("")` gönderir, tezgâh başlığı x=29, büyü ipucu Java'nın
`getFullname`i gibi (seviye gizleme, gri/kırmızı ad), menü ipuçları en son çizilir,
loom'da seçili düğmede ipucu yok, taş kesici kaydırması girdi sayısı değişince
da sıfırlanır, orta tıkla `CLONE` (yaratıcı), kırılmaz eşya grindstone'a girmez.

Bilinen sapmalar: harita görüntüsü, zırh askısı, büyülü kitap ve bayrak önizlemeleri
çizilmez; smithing/örs dönen simgeleri sadeleştirildi.

Örs sonucu (T51b, `client-parity`): `AnvilMenu.createResult` ve `onTake`
(`container/anvil.rs`): onarım malzemesi (`repairable`), iki aynı eşya,
kitap büyüleri (destek etiketi, `exclusive_set`, azami seviye, `anvil_cost`),
ön iş cezası, 40 sınırı, yeniden adlandırma (`setItemName`) ve eklenen eşyanın
tüketimi. `repairable` üretilmiş eşya tablosunda yok; `item/repairable.rs`
Java dökümünden alınan tablodur ve fixture ile her testte karşılaştırılır.
Oracle satırları `movement/container-clicks` (örs: onarım, iki alet, kitap,
desteklenmeyen büyü, dışlayan büyü, 40 sınırı, yaratıcı, ad). Bilinen sapma:
ad karşılaştırması istemcinin dilindedir ve bayat `infinite_materials`
yalnız sunucudan gelen `SetSlot`ta eskiyebilir.

## Tarif kitabı (envanter 2×2, çalışma tezgâhı, fırınlar)

Sunucu paketleri: `RecipeBookAdd` (74), `RecipeBookRemove` (75),
`RecipeBookSettings` (76), `PlaceGhostRecipe` (63); istemci paketleri:
`PlaceRecipe` (39), `RecipeBookChangeSettings` (46), `RecipeBookSeenRecipe`
(47, yazıldı, henüz gönderilmiyor). `RecipeDisplay` 5 türü ve `SlotDisplay` 11
türü okunur (istasyon alanı atılır). Durum `ClientWorld.recipe_book`
(bilinen tarifler, vurgular, dört kitap türünün açık/süzgeç ayarı, hayalet).
`photon-game/recipe_book.rs`: koleksiyonlar (kategori+grup, grupsuz tarif tek
başına), menüye uyan tarifler (`RecipeCollection.selectRecipes`), üretilebilirlik
(`StackedContents.canCraft`: çift parçalı eşleştirme, hasarlı/büyülü/adlı
yığınlar sayılmaz), `PlaceRecipeHelper.placeRecipe` (küçük tarifleri ortalar) ve
hayalet yuvalar. Arayüz `photon-render/gui/screen/recipe_book.rs`: düğme, panel
(arka plan, arama kutusu, süzgeç, sekmeler, 5×4 tarif düğmesi, sayfa okları),
sol tıkta `PlaceRecipe` (shift = `useMaxItems`, üretilemeyen son tarife tekrar
basmak yok sayılır), hayalet çizimi, `leftPos` kayması (`379` altı dar düzen
ayrı). Ayar değişikliği yerelde hemen uygulanır ve sunucuya gönderilir. Canlı:
envanter, tezgâh (8 tahtadan çubuk yerleşti) ve fırın.

Java kaynağına göre düzeltilenler (denetim sonrası): hayalet renkleri
(`0x30FF0000` altta, `0x30FFFFFF` üstte), envanterde küçük sonuç kutusu, boş
görüntülü hayalet yuvası atlanır, sonuç adedi, hayalet yalnız ekranı boyunca
yaşar, tarif yerleştirmede `lastPlacedRecipe` koruması (yerleştirilemezse tık
ekrana geçer), panel şeridi "dışarı" sayılmaz, sağ tık yerleştirmez, süzgeç
açıkken düğme yalnız üretilebilir tarifleri döndürür ve ikonlar aynı sonuçta
üst üste çizilir, sayfa yalnız sekme değişince sıfırlanır, arama sekmesi
kategorileri sırayla listeler, arama çevrilmiş ipucu satırlarında ve `ns:path`
ile yapılır, tarif/ok ipuçları (`moreRecipes`), `RecipeBookSeenRecipe`, dar
ekran (`<379`) yalnız arka planı çizer ve tıkları yutar, Esc dar kitabı kapatır,
Enter/Space son tarifi yeniden yerleştirir, T aramayı odaklar, tarif kitabı
`TrimPattern` holder okuması. `shapeless` hayaleti kaydıkça değil sırayla
yuvalara yerleşir.

Kaplama (`OverlayRecipeComponent`): çok tarifli düğmeye sağ tık kutuyu açar
(4 ya da 5 sütun, üretilebilirler önce, ekran merkezine göre kaydırma
`overlay_origin`), malzemeler 0,375 ölçekle 7 birimlik hücrelere çizilir, sol
tık yerleştirir, boşluğa tık kapatır; kapalı kitap kaplamayı götürür. Yeni
tarif olan düğme ve sekme 15 tikte sıkışır (`scale_about`), Ctrl basılıyken saat
durur, hayalet yuva ipucu ve `AnyFuel` için yakıtlı eşyalar (kayıt sırası)
eklendi.

Bilinen sapmalar: tost yok; animasyon ve döngü saati Java'da kare başına kısmi
tiktir (kare hızına bağlı), burada gerçek zamandır; `AnyFuel` sırası Java'nın
`FuelValues` kurulum sırası değil kayıt sırasıdır; kaplama ve düğme
ipuçlarının çizimi için oracle yok (eşya çizimi kaydedilmez), yalnız geometri
birim testli.

## Açık

Kap içeriği ipucu, kalan menü
türleri (beacon, brewing stand, lectern, tüccar, crafter) ve at envanteri.

## Simya standı

`MenuKind::Brewing` (menü türü 11): şişe yuvaları 0–2 (yalnız iksir, sıçrayan/kalıcı iksir ve cam şişe, hepsi 1'lik), bileşen 3 (`PotionBrewing.isIngredient`: eşya tanımındaki `brewing_ingredient` bayrağı, `BlockOracle`'daki istemcinin kendi `PotionBrewing.bootstrap`'ından üretilir), yakıt 4 (`brewing_fuel` etiketi). `quickMoveStack`, Java'nın yakıt yolundaki tuhaflığıyla (yakıta taşıma tıklamayı bitirir) oracle'da 3 satırla Java ile birebir. Ekran (`BrewingStandScreen`): arka plan, yakıt çubuğu, ok ve kabarcıklar `ContainerSetData` değerlerinden (0: kaynama süresi, 1: yakıt), boş yuva simgeleri; `container_screen` oracle'ında 7 satır. Canlıda bakılmadı.

## İşaret ışığı

`MenuKind::Beacon` (menü türü 9): tek ödeme yuvası (`beacon_payment_items` etiketi, 1'lik), `quickMoveStack` (tek eşya boş ödeme yuvasına) oracle'da 2 satırla Java ile birebir. Ekran (`BeaconScreen`, 230×219): dört kademenin güç düğmeleri, ikincil gücün ikinci seviyesi, onay ve iptal; düğmelerin açık/seçili durumu `ContainerSetData` seviyesinden (0) ve seçilen etkilerden (1, 2: etki kimliği + 1), ilk tikten sonra. Onay `SetBeacon` paketini gönderip kabı kapatır, iptal yalnız kapatır (`ScreenAction::{SetBeacon, Close}`). `container_screen` oracle'ında 10 satır Java ile birebir; tıklama davranışı birim testle (oracle fare tıklaması sürmüyor). Sapma: düğme ipucu gecikmesiz çıkar (Java yarım saniye bekler); yük eşyaları simgeleri (`graphics.item`) çizilmiyor. Canlıda bakılmadı.

## Crafter

`MenuKind::Crafter` (menü türü 7): 3×3 ızgara (0–8), oyuncunun 36 yuvası (9–44) ve sonra etkileşimsiz sonuç yuvası (45). Kapatılmış yuva (`ContainerSetData` 0–8 = 1) bir şey almaz (`CrafterSlot.mayPlace`), sonuç yuvasından alınmaz, `quickMoveStack` ızgara ↔ envanter; oracle'da 3 satır Java ile birebir. Ekran (`CrafterScreen`): kapalı yuva sprite'ı (eşya olsa da), yeşil/kırmızı redstone oku (veri 9: güç), kapıdaki açık boş yuvanın ipucu. Boş yuvaya tıklamak onu açıp kapatır (`ScreenAction::SlotState`: menü verisi hemen, `ContainerSlotStateChanged` paketi; birim testli, oracle tıklama sürmüyor); `container_screen` oracle'ında 6 satır birebir. Sapma: tıklama sesi çalınmaz; izleyici kipi bilinmez. Canlıda bakılmadı.

## Kürsü ve kitap görünümü

`MenuKind::Lectern` (menü türü 17): tek yuva (kitap) ve bir veri (sayfa), envanter yok; `quickMoveStack` hiçbir şey taşımaz. Ekran (`BookScreen`, `LecternScreen` + `BookViewScreen`): kitap dokusu, sayfa göstergesi (`book.pageIndicator`), sayfa metni (siyah, gölgesiz: `Style::shadow`; `font.split` 114 genişliğe, en çok 14 satır), sayfa düğmeleri (menünün sayfa verisine göre görünür), Done ve (yapma hakkı varsa) Take Book; düğmeler `ContainerButtonClick` 1/2/3 ya da kapatma, sayfa tuşları (PageUp/PageDown) aynı düğmeleri, metindeki `change_page` tıklaması `100+sayfa` düğmesini yollar. Kitabın sayfaları `written_book_content` ve `writable_book_content` bileşenlerinden (`Values::book_pages`). Oracle: `container_screen` 8 satır birebir (boş, kitap, ara ve son sayfa, uzun sayfa, düğme üstü, macera kipi). Sapma: `run_command` tıklaması ve sayfa çevirme sesi yok; sayfa, sunucunun veri güncellemesi gelince değişir (Java'daki gibi); metin üstü imleç ve ipucu yok. Canlıda bakılmadı.

## Tüccar (köylü, gezgin tüccar)

`MenuKind::Merchant` (menü türü 19): iki ödeme yuvası, sonuç yuvası, oyuncunun 36 yuvası. Teklifler `ClientboundMerchantOffersPacket`'ten (`MerchantOffer`, `ItemCost` ve bileşen koşulu: `photon_game::merchant`; paket çözümü `parse_merchant_offers_packet`), seviye ve deneyim `Trader`'da. Sonuç yuvasını istemci yapar (`MerchantContainer.updateSellItem`: seçilen teklif ipucuyla, stokta olmayan teklif sonuç vermez), sonucu almak ödemeyi düşürür (`MerchantResultSlot.onTake`, talep ve özel fiyatla artan/azalan ücret), `quickMoveStack`, çift tıklamada hiçbir yuvadan toplanmaz, `tryMoveItems` seçilen teklifin ödemelerini envanterden getirir (`ServerboundSelectTradePacket`, yerel menüye de yazılır). Oracle: `container-clicks` 7 satır, `container_screen` 9 satır (liste, kaydırıcı, deneyim çubuğu, fiyat üstü çizili, stok dışı) Java ile birebir; düğmeler bir kare sonra görünür (Java'daki gibi). Sapma: ticaret sesi (`getNotifyTradeSound`) ve köylünün `notifyTrade` etkileri yok; ipucu gecikmesiz; fare kaydırıcı imleci yok; tarif/deneyim sunucudan. Canlıda bakılmadı.


## Bileşen hash'leri (`HashedStack`, `client-parity`)

Tıklama paketi bir yığının eklediği her bileşeni, istemcinin
`decoratedHashOpsGenerator`ı gibi kalıcı codec'iyle `HashOps`a (CRC32C)
kodlayıp hash'ler; kaldırılanlar adıyla gider
(`photon-world/src/item_components/hash.rs`). Taşınan bileşenler: `custom_data`,
`max_stack_size`, `max_damage`, `damage`, `repair_cost`, `unbreakable`,
`custom_name`, `item_name`, `lore`, `rarity`, `enchantments`,
`stored_enchantments`, `enchantment_glint_override`, `dyed_color`,
`bundle_contents` (iç yığınların yamasıyla). Metin
bileşenleri ağdaki etiketten hash'lenir; `Style`/`NbtContents`'in boolean
alanları bayt değil boolean sayılır. Kalan bileşenler pakette yazılmaz
(sunucu yuvayı düzeltir). Java'nın kimlik haritası birden çok bileşeni rastgele
sırada yazar; Rust yamanın sırasını kullanır (sunucu sırayı okumaz), bu yüzden
paket baytı karşılaştırması tek bileşenli yığınlarla, çok bileşenli hash'ler
`movement/component-hashes` ile (bileşen adına göre) yapılır. Java bir yamayı
eşyanın varsayılanına göre sadeleştirir; ağ zaten sadeleşmiş yama taşır.

## Bundle, hasarlı eşya, zırh kilidi (`client-parity`)

`BundleItem`ın tıklama geçersiz kılmaları ve `BundleContents.Mutable`
(`photon-game/src/menus/container/bundle.rs`): taşınan bundle yuvadaki yığını
alır/birini bırakır, bundle üstüne taşınan yığın girer/biri imlece çıkar;
ağırlık kesirleri tam, boş içerik yamadan düşer. Sesler adıyla karşılaştırılır
(perde seviyenin tohumsuz rastgelesinden). `isStackable` hasar ve yamadaki
`max_stack_size`'ı okur. `prevent_armor_change` etkili büyülü zırh yaratıcı dışında
çıkmaz; hangi büyülerin bu etkiyi taşıdığı veri paketinden. Boş yama artık
her yerde bayt yok olarak tutulur (ağdaki `00 00` ile yerel yığın eşitlenir).
Seçili eşya (T59b): `BundleContents.selectedItem` yığının yanında tutulur
(telde yok, eşitlikte sayılmaz); ikincil tıklama seçileni çıkarır, ekleme
indeksi korur, birincil boş tıklama ve çıkarma seçimi düşürür. Ekranın
`BundleMouseActions`ı (`photon-render/src/gui/screen/bundle_mouse.rs`):
tekerlek `ScrollWheelHandler` adımlarıyla tersine sarar (gösterilen sayı:
en çok 12, 12'yi aşarsa 11, son satır dolu), fare yuvadan ayrılınca ya da
Shift/sayı tuşu tıklamasında seçim düşer ve her seferinde
`SelectBundleItem` (`-1` dahil, boş bundle'da da) gider. Oracle:
`movement/container-clicks`'e `SCROLL`/`UNHOVER` adımları ve
`bundle_selections` alanı, 5 satır. Açık: yaratıcı ekran ve ekran kapanışında
seçim düşürme yok.

İpucu görüntüsü (T59e): `ClientBundleTooltip` ve `GuiGraphics.tooltip`'in
metin + görüntü yerleşimi (`photon-render/src/gui/screen/bundle_tooltip.rs`):
sağ alttan dolan 24'lük yuvalar, seçili yuva vurgusu, 12'yi aşınca `+N`,
seçilenin adı üstte ayrı ipucunda, doluluk çubuğu (boş/dolu yazısı, dolu
sprite'ı), boş bundle açıklaması; bundle taşınırken de ipucu çıkar
(`showTooltipWithItemInHand`), yaratıcı ekranda da. Dokuz dilimli
sprite'larda kenar boyutun yarısıyla sınırlanır, sıfır boyutlu çizim
yapılmaz. Oracle: `gui/bundle-tooltip`, 10 satır.

## Taş çarkı sonucu (`client-parity`)

`GrindstoneMenu.computeResult` istemcide çalışır
(`photon-game/src/menus/container/grindstone.rs`): tek büyülü eşya lanetleri
dışında büyüsünü kaybeder (lanetsiz büyülü kitap düz kitap olur), aynı iki eşya
%5 dayanıklılık payıyla birleşir; onarım maliyeti kalan lanetlerden yeniden
hesaplanır. Bileşen yazımı `ItemStack.set/remove` gibi: eşyanın kendi değerine
eşit değer yamadan düşer (`components.rs`). Hangi büyünün lanet olduğu veri
paketinin `curse` etiketinden (`EnchantmentTraits`). Java tıklama paketinde
bileşenleri kimlik haritası sırasıyla yazar; oracle paketi tür kimliği sırasına
çevirir, Rust da o sırayla yazar (sunucu sırayı okumaz). Örs sonucu T51b'de.

## Eşya süsleri (`client-parity`)

`itemDecorations`'ın çubuğu ve bekleme örtüsü: `Item.getBarWidth/Color`
(hasar, HSV) ve bundle'ın doluluk çubuğu (`menus/container/bar.rs`), yuvalarda
ve hotbar'da (`GuiFrame::item_bar`, `item_cooldown`). `ItemCooldowns`
(`photon-game/src/item/cooldowns.rs`) `Cooldown` paketiyle dolar, oyuncu tikiyle
azalır; grup `use_cooldown` bileşeninden ya da eşya kimliğinden. Bekleyen eşya
kullanılamaz (`useItem`/`useItemOn`). Oracle: `gui/container-screen`
`decorations-*` satırları (Java'nın kaydettiği dolgular). Hotbar aynı çağrıyı
kullanır, ayrı oracle satırı yok.
