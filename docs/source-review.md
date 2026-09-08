# Kaynak incelemesi ve davranış sınırları

Referans: [26.2 envanteri](reference-baseline.md). Aşağıdaki yollar `.data/minecraft/source/` köküne göredir. Kod alıntısı yerine kendi ifadelerimizle gözlem ve kaynak konumları kaydedildi.

Bu ilk inceleme, bütün dosyaların satır satır incelendiği anlamına gelmez. Tüm kaynak ağacının envanteri çıkarıldı; ana döngü, zamanlayıcı, input, yerel hareket, çarpışma, hareket senkronizasyonu, kaynak paketi ve configuration geçişlerinin seçilmiş metot gövdeleri okundu. Diğer alt sistemlerde giriş noktaları ve metot haritası çıkarıldı. Her roadmap dilimi başlamadan kendi dalları ayrıca incelenecek.

## Dosya dağılımı ve istemci sınırı

| Kaynak alt ağacı | Java dosyası |
| --- | ---: |
| `net/minecraft/client` | 1.864 |
| `net/minecraft/client/renderer` | 701 |
| `net/minecraft/client/gui` | 444 |
| `net/minecraft/client/multiplayer` | 63 |
| `net/minecraft/client/player` | 9 |
| `net/minecraft/client/resources` | 101 |
| `net/minecraft/client/sounds` | 17 |
| `net/minecraft/network` | 411 |
| `net/minecraft/world/phys` | 28 |
| `com/mojang/blaze3d` | 211 |
| `com/mojang/realmsclient` | 127 |

Alt ağaç sayıları iç içedir, toplanmaz. `client/` tek başına istemci davranışını kapsamaz: `world/entity`, `world/item`, `world/inventory`, `world/phys`, registry, blockstate ve `server/packs` içindeki paylaşılan kod istemciye etki eder. Paket adında `server` olması otomatik dışlama ölçütü değildir.

Sunucunun otoriter dünya üretimi, mob AI, kayıt ve redstone yürütmesini Rust istemcisinde yeniden kurmayacağız. Bununla birlikte yerel hareket, blok/eşya tahmini, çarpışma şekilleri, client tick, ışık ve sunucudan gelen durumun sunumu istemcinin sorumluluğudur. Her paylaşılan sınıfın yalnızca erişilen istemci davranışı kapsamla ilişkilendirilmeli.

## Metot gövdelerinden çıkan kritik bulgular

| Kaynak konumu | Gözlem | Yol haritasına etkisi |
| --- | --- | --- |
| `client/DeltaTracker.java:38`, `Timer` | Oyun tick'i, gerçek zaman ve kısmi tick ayrı; pause/freeze farklı erişim davranışları var. | P02'de sabit FPS varsayımı kullanılmaz; zamanlayıcı ve döngü testleri ayrı. |
| `client/Minecraft.java:1151`, `runTick` | Kuyruktaki paketler ve görevler tick'lerden önce işleniyor; bir turda en çok 10 oyun tick'i yürütülüyor. Mouse birikimi ve render ayrı aşamalarda. | Timer'ın ürettiği tick sayısıyla yürütülen tick sayısı karıştırılmaz; sıralı olay izi gerekir. |
| `client/Minecraft.java:1219`, pause güncellemesi | Duraklama, yerel ve paylaşılmamış integrated server koşuluna bağlı. | Pumpkin'e bağlı istemcide menü açılmasının Java tek oyunculu duraklamasına eşit olduğu varsayılmaz. |
| `client/player/KeyboardInput.java:14,23` | Karşıt yönler birbirini sıfırlıyor; iki eksen birlikte normalize ediliyor. | P05'te kombinasyonları tabloyla test et; bağımsız eksen hızları varsayma. |
| `client/player/LocalPlayer.java:692,707,725` | Girdi, eşya kullanımı ve yavaş hareket çarpanlarından sonra kare hareket düzeltmesinden geçiyor. | Diyagonal hareketi yalnızca `Vec2.normalized` ile açıklamak eksik; P10 için ayrı referans. |
| `client/player/LocalPlayer.java:767` | Önceki input durumu, yeni input tick'i, sprint/flight pencereleri ve auto-jump etkileşiyor. | Basılı tuş snapshot'ı yeterli değil; tick'e bağlı tuşa basma/bırakma dizileri gerekir. |
| `client/MouseHandler.java:350` | Hassasiyet, smooth camera, dürbün ve eksen ters çevirme farklı yollar izliyor. | Mouse/kamera hissi P05 ve P14'te birikmiş delta ve frame zamanıyla sınanır. |
| `world/entity/Entity.java:1142,1245` | Step-up aday yükseklikleri ve eksen işleme sırası çarpışma sonucunu etkiliyor. | P09'da yalnızca AABB kesişimi değil, çözüm sırası ve hareket sonucu test edilir. |
| `world/entity/LivingEntity.java:2467` | Sürtünme, attribute'lar, yerçekimi ve chunk varlığı hava hareketine etki ediyor. | Eski sürüm sabitlerinden genel bir fizik modeli kurma; P10'da ilgili ortam dallarını ayrı ölç. |
| `client/player/LocalPlayer.java:227,263,303` | Değişen input ve sprint olayları; konum/rotasyon/durum paketleri; hareket eşiği ve periyodik konum hatırlatması var. | P11'de yalnızca son pozisyon değil gönderilen paket türü, alanları, sırası ve tick'i karşılaştırılır. |
| `client/multiplayer/ClientPacketListener.java:793` | Teleport durum güncellemesini, onay ve konum cevabını, prediction bilgisini etkiliyor. | Hareket düzeltmesi ve blok tahmini aynı senaryoda sınanmalı. |
| `client/multiplayer/ClientConfigurationPacketListenerImpl.java:80,97,147` | Registry/tags ve known pack seçimi, configuration'dan play'e geçişin parçası. | Kaynak/registry testleri ilk dünya girişinden önce gelir. |
| `client/resources/ClientPackSource.java:68` | JAR kaynakları ile haricî asset kökü aynı vanilla kaynak sağlayıcısında birleşiyor. | P04 asset seçimini iki katmanlı kaynak çözümlemesi olarak ele alır. |
| `network/VarInt.java:25,41` | Negatif değerler, devam bitleri, fazla uzun ve kesilmiş girişler gözlenebilir okuma/yazma davranışına sahip. | P03 yalnızca başarılı encode/decode roundtrip testlerinden oluşmaz. |
| `client/renderer/GameRenderer.java:376,393` ve `renderer/extract/LevelExtractor.java:92` | Render için durum çıkarma ve GPU çizimi ayrı giriş noktaları. | P15–P19'da durum/transform izi ile son görüntü karşılaştırması ayrı kanıtlar. Bu Rust mimarisi seçimi değildir. |
| `client/gui/Gui.java:145` | HUD, overlay ve screen çıkarımı koşullu ve sıralı. | P22'de aynı görüntünün yanında input yönlendirmesi, katman ve geçiş sırası test edilir. |
| `client/sounds/SoundEngine.java:340` | Kaynak seçimi, pitch/volume, attenuasyon, sessiz başlama, streaming ve kanal edinme davranışları var. | P20'de olay izi yeterli ilk katman; duyulan sonuç için ayrıca ses yakalama gerekir. |
| `world/level/levelgen/LegacyRandomSource.java:15-85`, `MarsagliaPolarGaussian.java:6-38`, `util/Mth.java:331-335` | `setSeed` 48 bitlik durumu yeniden kurup Gaussian önbelleğini temizliyor; `nextGaussian` eş sonucu önbelleğe alıyor; `fork` bir `nextLong` seed'i, positional factory ise koordinat/hash türevlerini kullanıyor. | P01.e'de seed sıfırlama, Gaussian, fork, positional ve default random yardımcılarının tüketim sıraları ayrı çağrı dizileriyle karşılaştırılır. |

Tablodaki `client/`, `network/` ve `world/` yolları `net/minecraft/` altındadır. Satır numaraları sabitlenen decompile ağacına aittir; gelecekteki sürüme taşınmaz.

## Diğer alt sistemlerin doğrulanan giriş noktaları

Bu bölüm metot/bağımlılık haritasıdır; bütün davranış dallarının incelendiği iddiası değildir.

| Alan | Kaynak ve giriş noktası | Bölüm |
| --- | --- | --- |
| Sayısal yardımcılar | `util/Mth.java`, `world/phys/Vec2.java`, `Vec3.java`, `AABB.java`, `core/BlockPos.java`, `Direction.java`; `util/RandomSource.java` | P01 |
| Wire ve protokol durumları | `network/FriendlyByteBuf.java`, `RegistryFriendlyByteBuf.java`, `protocol/*/*Protocols.java`, `Connection.java` | P03, P06–P07 |
| Kaynaklar ve pack sırası | `client/resources/IndexedAssetSource.java:createIndexFs`, `server/packs/repository/PackRepository.java:reload`, `server/packs/VanillaPackResources.java` | P04 |
| Ayarlar ve tuşlar | `client/Options.java`, `OptionInstance.java`, `KeyMapping.java`, `KeyboardHandler.java:keyPress` | P05, P22 |
| Login ve ortak mesajlar | `client/multiplayer/ClientHandshakePacketListenerImpl.java:handleHello/handleLoginFinished`, `ClientCommonPacketListenerImpl.java` | P06, P23 |
| Registry toplama | `client/multiplayer/RegistryDataCollector.java:collectGameRegistries`, `KnownPacksManager.java` | P07 |
| Dünya/cache | `client/multiplayer/ClientLevel.java:tick/tickEntities`, `ClientChunkCache.java:replaceWithPacketData`, `world/level/chunk/PalettedContainer.java` | P08 |
| Tahmin ve etkileşim | `client/multiplayer/MultiPlayerGameMode.java:startDestroyBlock/continueDestroyBlock/startPrediction/useItemOn`, `prediction/BlockStatePredictionHandler.java:endPredictionsUpTo` | P12 |
| Envanter | `world/inventory/AbstractContainerMenu.java:clicked/doClick`, `client/gui/screens/inventory/AbstractContainerScreen.java` | P13, P22 |
| Kamera | `client/Camera.java:tick/extractRenderState/calculateFov`, `client/renderer/GameRenderer.java:bobView/bobHurt` | P14 |
| GPU ve dünya çizimi | `com/mojang/blaze3d/{platform,opengl,vulkan,pipeline}`, `client/renderer/RenderPipelines.java`, `LevelRenderer.java:render`, `chunk/SectionCompiler.java` | P15–P16 |
| Model/atlas/texture | `client/resources/model/ModelManager.java:reload`, `model/sprite/AtlasManager.java`, `client/renderer/texture/TextureManager.java:tick/reload` | P15–P16 |
| Entity ve animasyon | `client/renderer/entity/EntityRenderDispatcher.java:extractEntity`, `LivingEntityRenderer.java:extractRenderState`, `client/model/`, `client/animation/` | P17 |
| Elde tutulan eşya | `client/renderer/ItemInHandRenderer.java`, `ScreenEffectRenderer.java` | P18 |
| Partikül | `client/particle/ParticleEngine.java:tick/extract`, `client/renderer/WeatherEffectRenderer.java` | P19 |
| Ses | `client/sounds/SoundManager.java:prepare/play`, `SoundEngine.java:tick/playDelayed`, `client/resources/sounds/` | P20 |
| Yazı | `client/gui/font/FontManager.java:reload`, `client/gui/Font.java`, `client/StringSplitter.java`, `client/resources/language/` | P21 |
| GUI | `client/gui/Gui.java`, `Hud.java`, `GuiGraphicsExtractor.java`, `render/GuiRenderer.java`, `screens/Screen.java` | P22 |
| Chat ve servisler | `client/multiplayer/chat/`, `network/chat/`, `client/gui/screens/social/`, `com/mojang/realmsclient/` | P23, P25 |
| Kalan içerik | `client/gui/screens/`, `client/renderer/special/`, `client/waypoints/`, `client/tutorial/`, `client/resources/` | P24–P25 |

`com/mojang/` ile başlayan yollar kaynak köküne, diğerleri `net/minecraft/` altına göredir. Dizin grupları kapsam keşfi için verilmiştir; her sınıfı aynı commit'te taşıma talimatı değildir.

## Karara bağlanması gereken eşitlik sınırları

- **Tek oyunculu oyun:** Java'nın dünya açma/oluşturma ve duraklatma akışı integrated server'a bağlı. Biz sunucu yazmayacağız. Yerel Pumpkin sürecinin bunu ne kadar karşılayabildiği P25'te ölçülecek. Dünya formatı, pause, LAN paylaşımı ve yönetim yeteneği eksikse istemci hilesiyle kapatılmayacak; açık eşitlik farkı olarak kalacak.
- **Sunucu mekanikleri:** Aynı kontrollü sunucu girdisinde Java ve Rust istemcilerinin eşitliği ile Pumpkin dünyasının vanilla sunucuyla eşitliği ayrı sorular. Pumpkin farkı istemci testlerini yeniden tanımlamaz.
- **Haricî hizmetler:** Online oturum, skin/cape, sosyal ekranlar ve Realms akışları yalnızca ekran görüntüsünden ibaret değil. Servis erişimi ve izin verilen otomasyon yolu ilgili bölümde incelenecek. Hazır olmaması bu alanları sessizce kapsam dışına çıkarmaz.
- **Platform ve render:** İlk yerel referans ortamı Linux x86_64. Grafik/ses backend'leri seçilmedi; Java'daki OpenGL/Vulkan klasörleri Rust için otomatik teknoloji kararı değildir. Yeni platformlarda ayrı referans ortamı gerekir.
- **Tam kapsam:** Birkaç demo senaryonun eşit olması tüm oyunun tamamlandığı anlamına gelmez. P24–P26'daki içerik, ekran ve protokol kapsam defteri kapanmadan tam eşitlik iddiası yapılmaz.
