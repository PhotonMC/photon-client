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
Java ile birebir. Tarif kitabı düğmesi çizilir ve basışı yutar, kitap açılmaz.
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

## Açık

Bileşenli eşyada bileşen hash'leri, kap içeriği ipucu, tarif kitabı, kalan menü
türleri (örs, büyü masası, taş kesici, loom, cartography, grindstone, smithing,
beacon, brewing stand, lectern, tüccar, crafter) ve at envanteri.
