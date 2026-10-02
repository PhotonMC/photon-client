# Kalan işler ve oynanabilir istemci yol haritası (2026-09-29)

> Sıradaki işler [görev panosundadır](tasks/README.md). Bu belge bugünkü durumu ve bilinen boşlukları kaydeder; görev bitince ilgili satır güncellenir.

Bu belge [roadmap](roadmap.md)ın P00–P26 tablosunu yeniden yazmaz; o tablo
bölüm kapsamını ve kabul kuralını tutar. Burası, bugünkü `play` modunun
gerçekte neyi yaptığını ve oynanabilir bir istemciye kadar **hangi işlerin,
hangi sırayla** kaldığını tek yerde toplar. Kabul kuralı değişmez: Java
referansı önce, kanıt sınıfı açık (`client-parity` / `server-conformance` /
canlı gözlem), orijinal içerik `.data/`da kalır.

Boyutlar kabadır: **S** birkaç saat, **M** bir iki gün, **L** birkaç gün,
**XL** bir hafta ve üstü.

## 1. Bugünkü durum

Çalışan: vanilla 26.2 sunucusuna offline bağlanma, chunk/ışık/blok
güncellemeleri, gökyüzü/bulut/hava/sis, yürüme-zıplama-sprint-eğilme-yüzme,
blok kırma ve koyma (tahmin ve ack dahil), eşya kullanma, HUD (can, yemek,
zırh, hava, deneyim, hotbar), eşya modelleri ve ikonları (özel modeller
dahil), eldeki eşya, hayatta kalma ve yaratıcı envanter ekranları
(tıklama tahmini, sürükleme, arama, ipucu, oyuncu modeli), üçüncü şahıs
oyuncu.

Kapsam ölçüsü (canlı oturumun paket işleyicisi, `crates/voxel-net/src/connection/play.rs`):

| Yön | İşlenen | Toplam | Eksik olanlar |
| --- | ---: | ---: | --- |
| Sunucu → istemci (play) | 33 | ~115 | kalan entity paketleri (`Animate`, `SetEquipment`, `EntityLink`…), chat/başlık/tablist, scoreboard/bossbar, ses, partikül, efekt olayları, blok entity verisi, harita, kap verisi, tarif kitabı, ilerlemeler |
| İstemci → sunucu (play) | 15 | ~60 | chat/komut, `PlayerInput`/`PlayerCommand`, `Interact`/`Attack`, pick item, kap düğmesi/ad değiştirme/takas, tabela/kitap, araç ve tekne paketleri |

Bugünkü kanıtın çoğu `server-conformance` (canlı sunucu paketi kabul etti) ve
canlı görsel gözlemdir; `client-parity` yalnızca sayısal/veri/tablo
katmanlarında (P01–P04, P09/P10 dalları, blok ve eşya tabloları, kap
tıklamaları) var. **Hiçbir bölüm `client-parity` ile kapanmadı.**

## 2. Kilometre taşları

Sıra bağımlılığa ve oynanabilirliğe göre; her taş bir öncekinin bıraktığı
açığı kullanır. Ayrıntılı iş listeleri sonraki bölümlerde.

| Taş | Ne kazandırır | Boyut | Bağımlılık |
| --- | --- | --- | --- |
| **M3.d Sunucunun açtığı kaplar** | Sandık, fırın, tezgâh, örs, dağıtıcı ve benzerleri | L | M3 (bitti), tarif tabloları |
| **M4 Entity'ler** | Diğer oyuncular, mob'lar, yerdeki eşya, saldırı/etkileşim | XL | Entity render iskeleti, M3 eşya çizimi |
| **M5 Metin ve chat** | Chat paneli/girişi, başlıklar, tablist, komut önerisi, scoreboard | L | Yazı motoru (bitti: stil, ipucu kutusu) |
| **M6 Ekranlar** | Duraklatma, ölüm, ayarlar, başlık, sunucu listesi, bağlantı kopması | L | M5 |
| **M7 Ses** | Ses tanımı, tetikler, konumsal miks | XL | Ses çıkışı ve çözücü kararı |
| **M8 Partikül ve efektler** | Kırılma, sıçrama, patlama, hasar, level olayları | L | M4, M7 (olay ortakları) |
| **M9 Sunucu uyumluluğu** | Çevrimiçi kimlik doğrulama, kaynak paketi, transfer, yeniden yapılandırma | XL | M6 |
| **M10 Eşitlik ve dağıtım** | Java karşılaştırması, platform matrisi, paketleme, performans | XL | hepsi |

Önerilen sıra: **M3.d → M5 → M6 → M4 → M8/M7 → M9 → M10**. Gerekçe: kap
ekranları envanter işinin doğal devamı ve yeni altyapı istemez; M5/M6
oyunun kullanılabilirliğini (chat, duraklatma, çıkış, ölüm) sağlar ve M4'ün
UI ihtiyaçlarını (isim etiketi, ölüm, oyuncu listesi) önceden kurar; M4 en
büyük iş olduğu için altyapısı oturduktan sonra başlamak daha az yeniden
yazım demektir. M7 (ses) ayrı bir bağımlılık kararı ister ve M4'ten
bağımsız yürüyebilir.

## 3. M3.d — Sunucunun açtığı kaplar

`OpenScreen` paketi zaten ayrıştırılıyor; ekran ve menü yok.

- **S** `ContainerSetData` (fırın ilerlemesi, büyü seviyeleri, işaret ışığı),
  `ContainerClose` çift yönlü kapanış, `ContainerSlotStateChanged`.
- [x] **M** Genel `AbstractContainerMenu` ve sandık/fıçı/ender sandığı/dağıtıcı/
  hopper/shulker kutusu ekranları ([kayıt](generic-container-conformance.md)).
- **M** (eski notu, yalnız kalan menüler) Genel `AbstractContainerMenu`: yuva düzeni ve kural sınıfı başına
  `mayPlace/mayPickup/quickMoveStack` (sandık, dispenser/dropper, hopper,
  shulker kutusu, at/eşek envanteri, ender sandığı, fıçı).
- **M** Kap ekranları: `ContainerScreen` (sandık ailesi), `DispenserScreen`,
  `HopperScreen`, `ShulkerBoxScreen`, `HorseInventoryScreen`.
- **L** Üretim ekranları: `CraftingScreen` + tarif kitabı çekirdeği,
  `FurnaceScreen` (fırın/ocak/dumanlayıcı) ve ilerleme okları.
- **L** Dönüştürme ekranları: örs (`RenameItem`), büyü masası (`ContainerButtonClick`,
  tohum ve seçenek metinleri), taş kesici, loom, cartography, grindstone,
  smithing, beacon (`SetBeacon`), brewing stand, kitaplık/lectern, tüccar
  (`MerchantOffers`, `SelectTrade`).
- **M** Tarif kitabı (`RecipeBookAdd/Remove/Settings`, `PlaceGhostRecipe`,
  `PlaceRecipe`), tarif tablosu için oracle.
- **S** Kalan envanter davranışı: bundle tıklama tahmini ve ipucu
  (`SelectBundleItem`), bileşen hash'li `HashedStack` (bileşenli eşyada
  sunucu düzeltmesi yerine tahmin), zırh kilidi büyüsü, hasarlı eşya
  (`damage` bileşeni takasta), sürükleme yuva sırası.
- **S** Eşya süsleri: dayanıklılık çubuğu, bekleme örtüsü (`Cooldown`), büyü
  parıltısı, kapsayıcı içerik ipucu (shulker/bundle satırları), harita
  ipucu, simya şablonu ve kitap ipucu satırları, `#etiket` araması (eşya
  etiketi tablosu), gelişmiş ipucu (F3+H).

Kanıt: kap tıklamaları için `movement/container-clicks` ailesinin
genişletilmesi (`client-parity`), ekranlar canlı gözlem.

### Kap menülerinde bilinen eksikler (unutulmasın)

- Kap menüsünde off hand bir yuva değil: off hand ile değiştirme (`SWAP` 40)
  tahmin edilmez, sunucunun düzeltmesi gelir; `Inventory.add` off hand'i de
  saymaz.
- Bileşenli eşyanın bileşen hash'leri yok: tıklama paketinde bileşeni olan yığın
  bileşensiz hash'lenir (sunucu düzeltir).
- Hasarlı eşya, büyü kilidi (zırh), bundle tıklama geçersiz kılmaları tahmin
  edilmez.
- Tarif kitabı çekirdeği var; kaplama, vurgu/tost, `SeenRecipe` ve dar ekran
  düzeni eksik (ayrıntı `generic-container-conformance.md`).
- Kap içeriği ipucu (shulker/bundle satırları) ve `#etiket` araması yok.
- Java menüleri girdi değişince sonucu istemcide de hesaplar (`AnvilMenu.createResult`,
  `GrindstoneMenu.createResult`, `LoomMenu.slotsChanged`): bizde sonuç yuvası
  yalnız sunucunun gönderdiği olur (taşınmadı). Anvil için onarım maliyeti, büyü
  birleştirme ve yeniden adlandırma tahmini L iş.
- Modellenmeyen menüler açılınca istemci hemen kapanış paketi gönderir.

## 4. M4 — Entity'ler

En büyük iş; altyapısı hazır değil. Dünya kopyasında entity durumu
(`ClientWorld`) `client-parity` ile ilerledi; canlı oturum `AddEntity`,
`RemoveEntities`, `MoveEntity*`, `TeleportEntity`, `EntityPositionSync`,
`RotateHead`, `SetEntityMotion`, `SetEntityData`, `SetPassengers` paketlerini
dünyaya uyguluyor (`connection/entities.rs`, T20; ayrıştırılamayan paket
bağlantıyı kesmez). Entity'ler her istemci tikinde ilerliyor (`ClientWorld::tick_entities`, yalnız `client-parity` ile doğrulanmış tik kolları; yaşayan entity'ler üç tikte yumuşak ilerler, mob gövdesi `BodyRotationControl` ile döner) ve çizim için `entity_snapshot` veriyor; uzaktaki oyuncular (varsayılan deriyle, ad etiketiyle) ile domuz ve inek (yetişkin/bebek, `temperate` çeşit: T24b) ve yerdeki eşyalar (bob/spin, yığın kopyaları; XP küresi: T25b) çiziliyor; model katmanları `voxel-data` tablosunda (358), çizim `Part::baked` ve tek paketlenmiş entity dokusuyla.

1. **Paketler (L)** Kalan: `SetEntityLink`, `SetEquipment`, `EntityEvent`
   (yalnız oyuncunun işlem düzeyi okunuyor), `Animate`, `HurtAnimation`,
   `DamageEvent`, `TakeItemEntity`, `MoveVehicle`, `MoveMinecart`,
   `PlayerLookAt/Rotation`, `SetCamera`, `PlayerInfoUpdate/Remove` eşlemesi.
   `Bundle`/`BundleDelimiter` ayrı ele alınmıyor: paketler sırayla işleniyor.

2. **Entity render iskeleti (L)** Render durumu çıkarımı, `EntityRenderer`
   sınıfları, model katmanları (`ModelLayers`, `LayerDefinitions`), gölge,
   isim etiketi, ışık ve çizim sırası; `player_model.rs` /
   `special_models.rs` üzerine `EntityModel` soyutlaması; entity texture
   atlas yolu; yürüme/saldırı/hasar/ölüm animasyonları (`KeyframeAnimation`).
3. **İçerik (XL)** Model sayısı yüzlerce: hayvanlar (~30), canavarlar (~40),
   su canlıları, tekne/minecart/ok/fırlatılan eşya, armor stand, item frame
   ve tablolar, yerdeki eşya (`ItemEntity` sallanma/dönme), XP küresi,
   uzaktaki oyuncular (deri indirme: profil dokusu için HTTP, yoksa varsayılan
   deri), zırh/eşya/pelerin/elytra katmanları, yakalar, binek eşyaları.
   Model başına oracle: eklem dönüşümleri `client-parity`.
4. **Etkileşim (M)** `Interact` (sağ tık entity), `Attack`, `PickItemFromEntity`,
   entity hedefleme (`ray` + `ProjectileUtil`), saldırı gücü/tepme, `PlayerCombat*`,
   ölüm/`Respawn` ekranına bağ.
5. **Binekler (L)** `PlayerInput`, `PlayerCommand`, `MoveVehicle`,
   `PaddleBoat`, at/domuz/tekne/minecart kontrolü, `MountScreenOpen`;
   P08'deki tekne/yolcu dalları üzerine.
6. **Birinci şahıs el (M)** Harita tutma, iki el, kullanım animasyonları
   (yeme, içme, yay, arbalet, kalkan, mızrak, fırça, trident), ekran
   efektleri (ateş, su altı, portal, balkabağı, hasar sallanması).

## 5. M5 — Metin ve chat

Bitti: `Component`/dil (`en_us`), stilli yazı, `EditBox`, ipucu kutusu.

- **Bitti (M5.a, çevrimdışı)** `SystemChat`, `DisguisedChat`, `PlayerChat`
  (imzalar atlanır, kısmi süzme tam süzme gibi değil geçer), sohbet türü
  süslemesi (`chat_type` kayıt defteri, veri paketinden), sohbet paneli (100
  satır, `StringSplitter` ile sarma, 180–200 tikte solma, etiket çubuğu),
  T ve `/` ile giriş kutusu, geçmiş (yukarı/aşağı), PageUp/PageDown, `Chat` ve
  `ChatCommand` gönderimi (imzasız, boş `LastSeenMessages`). Eksik: kaydırma
  çubuğu, güven simgesi, `DeleteChat`, eylem çubuğu (`overlay`), sohbet
  görünürlük seçeneğinin sunucuya bildirilmesi ve kısıtlı sohbet istemi, `show_item` hover ipucu, komut tıklamasında doğrulama/onay pencereleri, `chatLinks` seçenekleri
  olayları, `nextChatIndex` kopuşunda bağlantıyı kesme (yalnız uyarı).
- **L** İmzalı sohbet: `ChatSessionUpdate`, mesaj zinciri, `ChatCommandSigned`;
  çevrimdışı modda gerekmez, çevrimiçide zorunlu (M9).
- **Kısmen** Sekmeyle tamamlama: sunucudan `CommandSuggestion` (15) ile öneri
  alınır; liste Java ile birebir ([GUI oracle](gui-oracle.md): ilk öğe seçili,
  hayalet sonek, Tab/Shift+Tab/ok/Esc, 10 satırlık pencere, fare). Eksik:
  öneri ipuçları, oyuncu adı/özel sohbet tamamlamaları
  (`CustomChatCompletions`). Öneriler Java gibi istemcinin süzülmüş komut ağacından
  yerel üretilir (`command_suggest` oracle'ı, 143 satır): literaller,
  bool, kapalı kümeler (oyun modu, çapa, işlem, takım rengi, slot adı, şablon
  aynası/dönüşü, yükseklik haritası), takım ve oyuncu adları, seçiciler
  (`@p…`, `[` ve ad), koordinatlar (`~`/`^`, bakılan blok yok), süre
  birimleri, onaltılık renk örnekleri; yetkisiz komut önerilmez. Sunucuya
  Java gibi yalnız `ask_server`/bilinmeyen sağlayıcılı düğümlerde ve
  `objective`, `loot_*` türlerinde sorulur; Java'nın kayıt defterinden yerel
  listelediği türler (eşya, blok, parçacık, dimension, slot, ölçüt, kaynak,
  diyalog, bilinen yerel sağlayıcılar) ve seçici seçenekleri (`@a[`…) burada
  sunucuya sorulur (aynı değerler, farklı mekanizma; oracle'da
  `registry_backed` satırları). Seçici izni oyuncunun işlem düzeyinden gelir
  (varlık olayları 24–28, game master ve üstü seçici kullanır). Eksik:
  bakılan varlık UUID'si.
- **Kısmen** Komut ağacı (`Commands` paketi, `voxel-game/commands`): düğüm
  ağacı, Brigadier ayrıştırması (yönlendirme, aday sıralaması, istisnalar),
  renkli sözdizimi (`formatText`), kullanım ve hata satırları
  (`updateUsageInfo`, `getSmartUsage`); `command_parse` oracle'ı gerçek
  `CommandDispatcher`'a karşı 174 satır. Birebir okunan türler: Brigadier'in
  altısı, koordinatlar, kapalı kümeli kelimeler (oyun modu, çapa, işlem,
  slot, takım rengi, swizzle), tanımlayıcı, uuid, süre, varlık/ad. Şekle göre
  okunanlar (iyi biçimli metinde Java ile aynı, bozuk metinde sessiz):
  seçici seçenekleri, blok durumu/koşulu, eşya, NBT, bileşen, stil, parçacık,
  aralıklar, kaynak türleri; blok özellik doğrulaması ve kayıt defterine bağlı
  kontroller yok. Gönderim öncesi `verifyCommand` onay ekranı yok.
- **Bitti** Başlık/altyazı/eylem çubuğu (`SetTitleText`, `SetSubtitleText`,
  `SetTitlesAnimation`, `ClearTitles`, `SetActionBarText`, `SystemChat`
  overlay), `PlayerInfoUpdate/Remove`, `TabList` ve Tab ile tablist (sıralama,
  sütunlar, ping simgesi, üst/alt yazı; çevrimdışı kafasız). Tablist canlı
  denenmedi (birim testli). Eksik: çevrimiçi kafalar, takım biçimi, eylem
  çubuğunun renk animasyonu, eylem çubuğu canlı doğrulaması.
- **Kısmen** Duraklatma menüsü (`PauseScreen`, Esc): ızgara düzeni
  (`layout.rs`), düğmeler/simgeler/başlık (`widgets.rs`, `pause.rs`),
  `pause_screen` oracle'ı gerçek Java ekranına karşı 13 satır. Çalışan: Back
  to Game, Disconnect (pencereyi kapatır), Options (aşağıdaki ekran), Tab/Enter
  odak. Yapılmadı: Advancements, Statistics, hata bildirme, geri bildirim, arkadaşlar,
  oyuncu bildirme düğmeleri çizilir ama basınca bir şey olmaz (ekranları yok);
  arka plan bulanıklığı (`blurBeforeThisStratum`, GPU) yok; yerel sunucu
  düzeni (`hasSingleplayerServer`) yok; taslak rapor simgesi yok; ipuçları
  (simge düğmesi üzerinde bekleyince) var: `Holder`/`Positioner` (`tooltip.rs`, `frame.rs`), simge ipuçları pause'da; ok tuşlarıyla odak (`navigation.rs`) pause ve Options ekranında; ayar listelerinde ve dil listesinde Tab/Shift+Tab/ok gezintisi `OptionsList`/`language_select` içinde (T11b); Enter/Space Done ve başka ekran açan düğmelerde oracle'da basılmadı.
  Düğmeye sığmayan etiket saate göre kayar (`GuiFrame::millis`, `draw_label`);
  oracle saati sabitler (T07).
- **Kısmen** Seçenekler ekranı (`OptionsScreen`, pause → Options...):
  başlık, FOV kaydırıcısı (anında oyuna yansır), Online/dünya seçenekleri
  düğmesi, 10 düğmelik ızgara ve Done; Esc/Done `options.txt` kaydeder ve
  duraklatma menüsüne döner. `options_screen` oracle'ı gerçek Java ekranına
  karşı. Yapılmadı: alt ekranlar (Skin, Sounds, Video, Controls, Language,
  Chat, Resource Packs, Accessibility, Telemetry, Credits, Online, Difficulty)
  — düğmeler çizilir ama basınca bir şey olmaz; Telemetry çevrimdışı
  olduğundan etkisiz çizilir; ipuçları.
- **Kısmen** Video ayarları (`VideoSettingsScreen`, Options → Video Settings):
  kaydırmalı liste (`OptionsList`: çark, kaydırma çubuğu sürükleme, Java
  kırpması, odaktaki girdiyi görünüre kaydırma), bütün Java satırları çizilir
  ve düğmesi/kaydırıcısı değer tutar; yalnız **GUI ölçeği** çalışır (tıklama,
  Shift ile geri, Ctrl+tekerlek; pencere ve ekran yeni boyuta kurulur). Yapılmadı:
  görüş/simülasyon mesafesi, FPS sınırı, V-Sync, tam ekran ve çözünürlüğü,
  parlaklık, bulut/parçacık/mipmap/anisotropi/gölge/yaprak/saydamlık ve diğer
  grafik değerleri hiçbir şeyi etkilemez, değerleri `options.txt`'e yazılmaz ve
  ekran kapanınca unutulur; grafik ön ayarı yalnız "Custom"a döner (ön ayar
  seçince diğer değerleri ayarlamaz); `applyValueImmediately=false`
  kaydırıcılar (görüş mesafesi vb.) 600 ms gecikmeden değil bırakınca oturur;
  ipuçları, tam ekran çözünürlüğü listesi
  (monitör yok), sahte GPU uyarısı ekranı yok. `video_settings_screen`
  oracle'ı gerçek Java ekranına karşı (120 satır).
- **Kısmen** Ölüm ekranı (`DeathScreen`, `PlayerCombatKill`, `deathScreen.*`):
  gradyan, başlık/neden/skor (yerel oyuncunun `DATA_SCORE_ID` verisi), "Respawn"
  ve "Title Screen" düğmeleri (20 tik sonra uyanır), hardcore metinleri,
  anında yeniden doğma kuralı (`immediate_respawn`, oyun olayı 11);
  `death_screen` oracle'ı gerçek Java ekranına karşı 14 satır. Canlı:
  öldür → ekran → Respawn → yeni can → ekran kapanır. Artık otomatik
  yeniden doğma yok. Yapılmadı: "Title Screen" hardcore değilse önce
  `ConfirmScreen` (`deathScreen.quit.confirm`, 20 tik gecikme; hayır = yeniden
  doğ) ister, evet bağlantıyı keser (başlık ekranı yok), taslak rapor simgesi, ölüm nedeninin tıklanabilir bağlantıları,
  ölü oyuncu olarak girişte ekran (seviye yüklenemiyor).
- **Kısmen** Bağlantı kopması ekranı (`DisconnectedScreen`): `Disconnect`
  paketinin nedeni, okuma hatası (`Internal Exception: …`) ve akış sonu
  (`disconnect.endOfStream`) ekrana gelir; bağlantı bitince hiçbir şey
  gönderilmez. `disconnected_screen` oracle'ı gerçek Java ekranına karşı 9
  satır. Yapılmadı: "Back to Server List" yalnız çıkar (sunucu listesi/başlık
  ekranı yok), arka planda başlık panoraması ve bulanıklık yok (yalnız menü
  arka planı dokusu), yapılandırma aşamasındaki kopma ekranı (giriş sırasında
  kopma hâlâ hata ile çıkar), hata bildirim bağlantısı/raporu düğmeleri,
  `timeout` ve `closed` gibi özel ileti anahtarları.
- **Bossbar** (`BossEvent` paketi, `BossHealthOverlay`, ilerleme yumuşatması)
  yapıldı, `boss_bar` oracle'lı. Eksik: seviye değişince çubukları silmek,
  koyulaştırma/sis/müzik bayrakları yalnız durum (ses ve sis yok).
- **Scoreboard** (`SetObjective`, `SetScore`, `ResetScore`,
  `SetDisplayObjective`, `SetPlayerTeam`; `voxel-game/scoreboard.rs`) yapıldı:
  kenar çubuğu (takım rengine özel dahil), tablist puan sütunu ve kalpler,
  takım öneki/soneki/rengi ve takım sırası; `sidebar` ve genişletilmiş
  `tab_list` oracle'lı. Eksik: `below_name` (oyuncu adı altı puan, entity
  çizimi gerekir), takımın ad etiketi/çarpışma/dost ateşi alanları yalnız
  okunur (kullanım yok), kalp sütununun yanıp sönmesi birim testli ama
  oracle'da yalnız ilk çizim, bozuk (`unknown objective`) paketler sessizce
  yok sayılır.
- **M** Kalan ekran metinleri: yok (`TrackedWaypoint` T14'te yapıldı).
- **M** Yazı motoru kalanı: unifont yedeği (Latin dışı glifler), altı çizili /
  üstü çizili / karışık yazı, sağdan sola metin (bidi), tıklama/üzerine gelme
  olayları (`ClickEvent`, `HoverEvent`), yazı tipi seçimi, gölge rengi,
  dil değiştirme ve yeniden yükleme (Türkçe dahil), `IME` girişi.
- **S** Metin girişi: fareyle konumlama, çift tıklamayla kelime seçme, seçim
  vurgusu (`GUI_TEXT_HIGHLIGHT` mantıksal işlemi).

## 6. M6 — Ekranlar ve widget'lar

- **M** Widget kitaplığı: düğme, kaydırıcı, onay kutusu, liste, sekme,
  odak/gezinme, fare tıklaması ve kaydırma katmanı (`P22.a`),
  ekran yığını ve `setScreen` geçişleri.
- **M** Duraklatma menüsü (Esc), bağlantıyı kesme, ayarlara giriş; şu an
  Esc yalnız imleci bırakır ve ikinci Esc pencereyi kapatır.
- **M** Ölüm ekranı (`DeathScreen`, yeniden doğ / başlığa dön); bugün ölünce
  otomatik yeniden doğuyor.
- **L** Ayarlar: ses, video (görüş mesafesi, FOV, GUI ölçeği, bulanıklık),
  kontroller (tuş atamaları — `P05.a/b`), erişilebilirlik; ayarların
  kaydı/yüklenmesi. **Kısmen:** `options.txt` okunur/yazılır (`voxel-game/src/
  options.rs`: FOV, GUI ölçeği, fare, tekerlek, sohbet değerleri; bilinmeyen
  anahtarlar korunur); FOV, GUI ölçeği, duyarlılık, ters çevirme ve tekerlek
  oyuna bağlı; sohbet değerleri (genişlik, ölçek, opaklıklar, satır aralığı, yükseklikler) sohbet çizimine bağlı (`ChatSettings`).
- **Kısmen** Dil seçimi (`LanguageSelectScreen`, Options → Language...): paketin dil listesi (`LanguageSource`: jar `en_us` + asset indeksindeki `lang/*.json`, `en_us` üstüne seçili dil), arama, seçim, Done/Enter/çift tık dili değiştirir (`options.txt` `lang`, `Gui::set_language`; açık pause/options ekranları yeni dille kurulur). Yapılmadı: `unifont` yedeği (Türkçe ş/ğ/İ/ı, Arapça, CJK… çizilmez), sağdan sola şekillendirme, Font düğmesi ekranı, ok tuşlarıyla liste gezinme (T11). `language_screen`/`lang_text` oracle'ları; dil değişince sohbet geçmişi yeni dille yeniden çözülür (silinmiş-mesaj işareti hariç).
- **Kısmen** Kontroller ve fare ayarları (`ControlsScreen` → `MouseSettingsScreen`, Options → Controls...): duyarlılık, tekerlek duyarlılığı ve ters çevirmeler `Options`'a yazılır, kameraya ve tekerleğe anında yansır (`turn_by`); ayrık kaydırma, imleç değişimi, Hold/Toggle, otomatik zıplama, koşu penceresi, operatör sekmesi yalnız ekranda yaşar; Key Binds ekranı yok (düğme çizilir, basınca bir şey olmaz); ham fare girdisi satırı yok (macOS'ta desteklenmez). `mouse_settings_screen` ve `controls_screen` oracle'ları.
- **Kısmen** Sohbet ayarları (`ChatOptionsScreen`, Options → Chat Settings): 18 satır, Java etiketleri; görünürlük, genişlik, ölçek, opaklık, arka plan opaklığı, satır aralığı ve yükseklikler `Options`'a yazılır ve sohbeti anında yeniden sarar (`rescaleChat`). Yapılmadı: renkler/bağlantılar/öneriler/gecikme/imzalı sohbet vb. değerler yalnız ekranda yaşar (taslak kaydı `saveChatDrafts` olarak `Options`'ta); anlatıcı yok (düğme uykuda); görünürlüğün sunucuya bildirilmesi ve kısıtlı sohbet istemi; Tab/ok odağı (T11). `chat_options_screen` oracle'ı gerçek ekrana karşı.
- **L** Ana menü, sunucu listesi ve doğrudan bağlanma, bağlantı/yükleme
  ekranları (`LevelLoadingScreen`, `ReceivingLevelScreen`), bağlantı kopma
  ekranı, dünya seçimi (`P25`: Pumpkin ile eşleme).
- **M** Kitap (`OpenBook`, `EditBook`), tabela düzenleme (`OpenSignEditor`,
  `SignUpdate`), komut bloğu ve yapı bloğu ekranları, jigsaw.
- **M** Ilerlemeler (`UpdateAdvancements`, `SelectAdvancementsTab`), istatistik
  (`AwardStats`), sosyal ekran, toast'lar, kredi/`Poem`.
- **S** Kalan HUD: binek canı, `AttackIndicator`, kavşak nişangâhı ve
  hata ayıklama katmanı (F3), ekran görüntüsü (F2), F1 HUD gizleme, F5
  kamera (var), zoom.

## 7. M7 — Ses

- **Karar (S)** Ses çıkışı ve Ogg/Vorbis çözücü için kaynak incelemesi ve
  bağımlılık kararı ([roadmap](roadmap.md#oynanabilir-istemci-sırası-2026-09-29)
  tek üretim bağımlılığı kuralı: `flate2`).
- **M** Ses tanımı (`sounds.json`), ağırlıklı seçim, `Sound`/`SoundEntity`/
  `StopSound` paketleri, kategori ve ses ayarları.
- **M** Oyun tetikleri: adım, blok kırma/koyma, eşya kullanma, hasar,
  ortam sesi ve müzik (`MusicManager`), yağmur.
- **L** Konumsal miks: dinleyici, mesafe sönümü, pitch, `AudioStream`
  akışı, pause sırasında devam eden kategoriler, altyazı bağlantısı.
- Kanıt: kaynak seçimi ve zaman damgaları `client-parity`; duyulan ses
  ayrıca yakalanır.

## 8. M8 — Partikül ve efektler

- **M** `LevelEvent` (kapı, fırın, tuğla kırılma, portal…), `LevelParticles`,
  `Explode`, `BlockEvent` (nota bloğu, sandık kapağı, piston), `BlockEntityData`
  (tabela metni, bayrak deseni, kafa profili, sandık/çömlek/kova).
- **L** Partikül sistemi: doğum, random tüketim sırası, hareket/çarpışma,
  yaşam, çizim sınıfları, sprite atlası; blok kırılması, yağmur sıçraması,
  efekt ve sürükleme partikülleri.
- **M** Blok entity çizicileri: yerleştirilmiş sandık, shulker, tabela,
  bayrak, kafa, yatak, çan, ender portalı, beacon ışını, spawner, harita
  çerçevesi, kitaplık kitabı, tezgâh kitabı (`BookModel`).
- **S** Efekt ekran örtüleri (hasar, gece görüşü, karanlık, körlük),
  `DamageEvent` sallanması, `EntityEvent` tetikleri.
- **S** Bozuk kalanlar: gece görüşü/`Darkness` etkisi ve patron sisi yok
  (sis ve ışık), dünya sınırı çizimi (`InitializeBorder`, `SetBorder*`
  paketleri canlı oturumda işlenmiyor), su altı sisi ve perde.

## 9. M9 — Sunucu uyumluluğu

- **XL** Çevrimiçi mod: `Encryption`, Mojang oturum sunucusu kimlik
  doğrulaması, hesap belirteci yönetimi (bugün yalnız offline). Hesap
  gerektiren gerçek servis kontrolleri erişim yoksa engelli kaydedilir.
- **M** Yeniden yapılandırma (`StartConfiguration` ve play → configuration
  → play), yeniden bağlanma, `Transfer`, `Cookie` paketleri.
- **L** Sunucu kaynak paketi (`ResourcePack` push/pop, indirme, doğrulama,
  reload, kullanıcı onayı ekranı, pack sırası) — `P04.c` reload akışı.
- **M** Diyalog paketleri (`ShowDialog`), özel yük (`CustomPayload`) ve marka.
- **S** Pumpkin sürümüyle karşılaştırma koşuları (aynı senaryo, iki sunucu).

## 10. Görüntü ve simülasyon borçları (paralel)

- **L** `P15.a` Java kare karşılaştırması: kontrollü Java kareleri,
  değişkenlik ölçümü, piksel toleransı; canlı kayıttaki "referans karelerle
  karşılaştırılmadı" kalemlerinin bağlanması.
- **M** Görünürlük: frustum ve mağara ayıklaması, bölüm içi yarı saydam
  sıralama, bölüm derleme sırası (`SectionCompiler`), P02.b kare
  zamanlayıcısı.
- **M** Işık/görüntü: yumuşak gölge farkları, `AmbientOcclusion` kalanları,
  bulanıklık, `CardinalLighting` (nether), `LightTexture` gece görüşü.
- **M** Hareket kalanları: uçuş/elytra/auto-jump/araç (P10.d/e),
  attribute/effect etkileşimleri, dimension geçişi ve portal, `PlayerAbilities`
  (yaratıcı uçuş çift-boşluk), teleport/relative düzeltmeleri (P11.c).
- **M** `P14` kamera sırası: tick, suya giren kamera sıvı kontrolü, üçüncü
  şahıs kamera çarpışması, hassasiyet yumuşatma ve ters eksen.
- **S** `P12` kalanları: iki el önceliği, gecikmiş ack, hit sonucu
  animasyonları ve sesleri.
- **S** Kapsam defteri (`P24`): kayıt defterlerindeki her blok/eşya/entity
  için "çizilir mi, tıklanır mı, sesi var mı" otomatik denetimi; atanmamış
  davranışlar görünür listede kalır. Bugün eksik olduğu bilinenler:
  trident özel modeli, heykellerin diğer duruşları, Noel sandığı dokusu,
  bayrak/kalkan desenleri, süslü çömlek süsleri, çift sandık, animasyonlu
  özel modeller (açılan kapak, pistonlar, sandık kapağı).

## 11. M10 — Eşitlik ve dağıtım

- **XL** `client-parity` kapanışları: bölüm başına Java–Rust karşılaştırmalı
  kayıt (P08, P12, P13, P16, P17, P22); `server-conformance` kayıtlarının
  `client-parity`e yükseltilmesi ve kapanmayan bölümlerin görünür listesi.
- **L** P26.a/b: aynı kontrollü oturumun Java ve Rust'ta çalıştırılması,
  Pumpkin'de iki istemci senaryosu (bağlan→chunk→hareket→eylem→envanter→
  ses/UI→respawn→yeniden bağlan).
- **L** Platform matrisi: Linux ve Windows (GLFW/Vulkan), macOS
  MoltenVK notları (`docs/local-setup.md`); Java 25 kurulumu kalanları.
- **M** Performans: tablolar hesaplamaya döndü, açılış ~375 ms, ayak izi
  643 → 513 MB, section vertex'i 28 bayt ([yol haritası](architecture-roadmap.md));
  ışık kuyruğu sınırı (D1) yapılmayacak; kalan: entity sayısı arttığında çizim
  (M4 ile).
- **M** Paketleme: özel asset kökü seçimi (P04.d), orijinal içeriğin pakete
  girmemesi, yürütülebilir dosya dağıtımı, sürüm ve kaydedilebilir ayarlar.

## 12. Kod sağlığı

- Kod düzeni, crate katmanları ve karmaşıklık sınırları
  [kodlama standartlarındadır](coding-standards.md); kalan istisnalar
  gerekçeli `#[expect]` ile işaretlidir ve bölündükçe kaldırılır.
- **S** Oracle harness: Java tekrarlarının determinizm denetimini yaygınlaştırma
  (bileşen sırası bu oturumda yakalandı: `DataComponentMap` kimlik hash'iyle
  dolaşıyor); macOS arm64'te `math` ailesinin çalıştırılması.
- **S** Test envanteri: canlı sunucu betiklerinin (`VOXEL_SCRIPT`) bir
  senaryo dosyası biçimine ve sürekli çalıştırmaya alınması.

## 13. Sıradaki işler

Görevler [görev panosundadır](tasks/README.md) (T01–T26 ve PLAN görevleri); burada ikinci bir liste tutulmaz.

## 14. Mimari ve kod sağlığı kuyruğu

Hepsi sapmasız; oyun özelliklerinin arasında, ölçerek yapılır.

- **M** Büyük dosyaları böl: `voxel-render/src/vulkan.rs` (~4,2k satır: aygıt
  ve swapchain, arena ve staging, boru hatları, sahne çizimi), `voxel/examples/
  behavior_adapter.rs` (~18k satır, aileye göre modüller), `voxel-world/src/
  lib.rs` ve `tests.rs`. vulkan.rs bölünmesi §15'in ön koşulu.
- **S** `u16` indeks: 65.535'ten az köşeli section'lar (~10 MB).
- **M** Bellek: ayak izi 513 MB; kalan başlangıç tepesinden (atlas kurulumu,
  çözülen PNG'ler, `pending` durum listeleri). `footprint` ile önce/sonra.
- **S** Canlı duman testi: `VOXEL_SCRIPT` senaryolarını dosya biçimine alıp
  `tools/voxel.py` altında tekrarlanabilir koşu (katıla → envanter →
  yeniden boyutlandırma → ekran görüntüsü).
- **S** Küçük temizlik: `expect` gerekçeleri, kullanılmayan üretici/fixture.
- Yapılmayacak: D1 ışık kuyruğu sınırı; ışık yayılımını paralelleştirme.

## 15. Vulkan güvenlik denetimi (düşük öncelik)

`vulkan.rs` 63 `unsafe` taşır; `// SAFETY:` açıklaması ve çalışma zamanı
denetimi yok. Makinede Khronos doğrulama katmanı kurulu
(`/usr/local/lib/libVkLayer_khronos_validation.dylib`).

1. **S** `VOXEL_VULKAN_VALIDATION=1`: `create_instance`'a
   `VK_LAYER_KHRONOS_validation` ve `VK_EXT_debug_utils`, mesaj geri çağrısı
   stderr'e yazar; `=strict` ilk mesajda süreci sonlandırır; senkronizasyon
   doğrulaması `VK_EXT_validation_features` ile. Kapalıyken maliyet yok.
2. **S** Doğrulamalı duman testi: §14'teki senaryolar `strict` ile, kabul
   ölçütü sıfır mesaj; bulunanlar ayrı commit'lerle düzeltilir.
3. **M** Kesin tipler: `Buffer`+`DeviceMemory` çiftleri için `Drop`'lu
   sarmalayıcılar (arena bloğu, staging parçası, okuma tamponu); yeni
   bağımlılık yok.
4. **S** `clippy::undocumented_unsafe_blocks` uyarısı ve her `unsafe` bloğa
   `// SAFETY:`; kural `coding-standards.md`'ye.
5. Sınır: doğrulama katmanı yalnız koşulan senaryoları denetler; Miri FFI'yi
   çalıştıramaz.
