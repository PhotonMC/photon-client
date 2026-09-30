# Sunucunun açtığı genel kaplar (sandık, fıçı, ender sandığı, dağıtıcı, hopper, shulker kutusu, çalışma tezgâhı)

Kapsam: `OpenScreen` ile açılan `generic_9x1..9x6`, `generic_3x3`, `hopper` ve
`shulker_box` menüleri. Fırın, tezgâh, örs gibi diğer menü türleri henüz
modellenmedi; sunucu bunlardan birini açarsa istemci kapanış paketini hemen
gönderir (sunucu görünmeyen bir pencereyi beklemesin).

## Menü modeli

`voxel_game::container_menu::ContainerMenu` artık oyuncunun kendi menüsü ile
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
temizleme). Sonuç yuvalarının içeriğini sunucu üretir; istemci girdi değişince
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

Bilinen sapmalar: bundle/hasarlı eşya/zırh kilidi tahmin edilmez; bileşen
hash'i yok; harita görüntüsü, zırh askısı, büyülü kitap ve bayrak önizlemeleri
çizilmez; smithing/örs dönen simgeleri sadeleştirildi; örste onarım malzemesi
tüketimi tahmin edilmez.

## Tarif kitabı (envanter 2×2, çalışma tezgâhı, fırınlar)

Sunucu paketleri: `RecipeBookAdd` (74), `RecipeBookRemove` (75),
`RecipeBookSettings` (76), `PlaceGhostRecipe` (63); istemci paketleri:
`PlaceRecipe` (39), `RecipeBookChangeSettings` (46), `RecipeBookSeenRecipe`
(47, yazıldı, henüz gönderilmiyor). `RecipeDisplay` 5 türü ve `SlotDisplay` 11
türü okunur (istasyon alanı atılır). Durum `ClientWorld.recipe_book`
(bilinen tarifler, vurgular, dört kitap türünün açık/süzgeç ayarı, hayalet).
`voxel-game/recipe_book.rs`: koleksiyonlar (kategori+grup, grupsuz tarif tek
başına), menüye uyan tarifler (`RecipeCollection.selectRecipes`), üretilebilirlik
(`StackedContents.canCraft`: çift parçalı eşleştirme, hasarlı/büyülü/adlı
yığınlar sayılmaz), `PlaceRecipeHelper.placeRecipe` (küçük tarifleri ortalar) ve
hayalet yuvalar. Arayüz `voxel-render/gui/screen/recipe_book.rs`: düğme, panel
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

Bilinen sapmalar: sağ tıkla çok tarifli koleksiyon kaplaması yok (sağ tık
yerleştirmez, yalnız yutulur); yeni tarif düğmelerinin ve sekmelerin sıkışma
animasyonu, tost ve zamanlayıcının Ctrl ile durması yok; hayalet yuva ipucu yok;
`AnyFuel` hayaleti boş (yakıt listesi yok); tarif kitabının çizimi için oracle
yoktur (yalnız Java kaynağından okunup birim testlenir).

## Açık

Bileşenli eşyada bileşen hash'leri, kap içeriği ipucu, kalan menü
türleri (beacon, brewing stand, lectern, tüccar, crafter) ve at envanteri.
