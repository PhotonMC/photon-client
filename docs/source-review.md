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
| `world/level/levelgen/structure/BoundingBox.java:48-64,288-318` | `BoundingBox` constructor'ı ters int endpoint'lerini eksen bazında normalize ediyor; `AABB.of(BoundingBox)` min uçları doğrudan taşıyıp max uçlarına Java int `+1` uyguluyor. | P01.d AABB BoundingBox köprüsü; int sınırları ve taşma ayrı senaryolarla doğrulanır. |
| `world/phys/AABB.java:22-29,59-85,87-94,129-460,463-467`, `world/phys/BlockHitResult.java:6-65`, `world/phys/shapes/Shapes.java:44-377,379-466`, `world/level/block/state/properties/AttachFace.java:5-8`, `VoxelShape.java:29-121,163-182,184-200,202-235,252-307`, `BitSetDiscreteVoxelShape.java:112-158`, `core/Direction.java:311-328` | AABB uçlarının normalizasyonu, contract delta'sının tek endpoint'e uygulanması, inflate delta'sının iki endpoint'e uygulanması, intersect ortak endpoint'lerinin `Math.max`/`Math.min` ile seçilmesi, ham intersects endpoint'lerinin normalize edilmeden karşılaştırılması, Vec3 endpoint'lerinin Math.min/Math.max ile normalize edilmesi, BlockPos unit box'ında int +1 taşmasının korunması ve Vec3 contains sınırlarının iletilmesi, collidedAlongVector'ın merkez segmenti, aday kutu inflate'ı, contains/clip kontrolü ve liste taraması, minmax kapsam endpoint'lerinin `Math.min`/`Math.max` ile seçilmesi, hasNaN'ın altı endpoint'i Double.isNaN ile taraması, getCenter/getBottomCenter midpoint'lerinin Mth.lerp ile üretilmesi, getMinPosition/getMaxPosition Vec3 sonuçları, Axis.choose ile min/max endpoint seçimi, tek endpoint setter'larının diğer beş endpoint'i koruyarak constructor normalizasyonu yapması, `expandTowards(Vec3)` overload'ının Vec3 bileşenlerini scalar yola aktarması, scalar move çağrısında her endpoint'e eksen bazında delta eklenmesi, BlockPos overload'unda i32 koordinatların, Vec3 overload'unda f64 bileşenlerin ve Vector3f overload'unda f32 bileşenlerin scalar taşıma yoluna aktarılması, üç eksen uzunluğunun çıkarılması ve ortalama `getSize` hesabı, distanceToSqr için eksen başına dış boşluk seçimi ve Mth.lengthSquared toplama sırası, `ofSize` için merkezden yarım boyut çıkarıp ekleme ve constructor normalizasyonu, AABB kutu kurucuları/üreticileri, box üretimi, Shapes.create(AABB)'nin normalize edilmiş AABB uçlarını create(double...) yoluna aktarması, Shapes.create(double...)'ın eşik, grid ve ArrayVoxelShape yolları, Shapes.INFINITY'nin sınırsız box olarak kurulması, Shapes.EPSILON ve BIG_EPSILON sabitlerinin eşik değerleri, çok küçük aralıkların empty olması, koordinat düzlemlerinin taşınması, ham endpoint'li koordinat clip overloadunun eksen taraması, iterable clip'in AABB'leri BlockPos ile taşıması, ortak en yakın ölçek ve yüz yönünü koruması, `BlockHitResult` hit alanlarının snapshot'lanması, VoxelShape clip'in kısa segment ve dolu hücre hızlı yolunu AABB fallback'inden ayırması, Direction approximate-nearest seçimindeki float dönüşümü ve enum sırasının yön sonucunu belirlemesi, BooleanOp sonuçlarının dolu hücrelerden kutulara çıkarılması, shape hücrelerinin hareketli AABB ile eksen taraması ve iterable shape distance güncellemesi, closestPointTo'nun empty/box clamp/strict tie davranışı ile getFaceShape'in axis sign/slice/block sadeleştirmesi, Shapes.blockOccludes'in sınır kontrolü ile yüz BooleanOp operand sırası, mergedFaceOccludes/faceShapeOccludes'in unit block coverage davranışı ve Shapes.equal'in NOT_SAME birleşimini tersleyerek geometri eşitliği sorgulaması, VoxelShape.min/max'ın axis başına firstFull/lastFull sınır düzlemlerini ve empty sonsuzluklarını döndürmesi, VoxelShape.min/max(axis,b,c)'nin AxisCycle ile sütun seçimi ve dış sütun sonsuzluklarını döndürmesi, VoxelShape.move(Vec3/Vec3i)'nin scalar move yoluna yönlendirilmesi ve empty shape kısa yolunun korunması, VoxelShape.optimize()'ın forAllBoxes ile kutu yeniden kurulumu, forAllBoxes callback sırası, getCoords coordinate plane sonuçları ve forAllEdges'in üç AxisCycle kenar taraması, VoxelShape.singleEncompassing'in boş shape'i empty'e, dolu shape'i min/max sınır box'ına indirgemesi ve Shapes.or varargs yolunun soldan sağa OR azaltımı ile Shapes.rotate'ın signed eksen permütasyonu ve merkez yansıtması, merkez overload'ının özel merkez bileşenlerini output eksenlerine taşıması ve Shapes.rotateHorizontalAxis'ın X/Z EnumMap türetmesi ve merkez overload'ının özel merkezi X dönüşüne aktarması ve Shapes.rotateAllAxis'ın X/Y/Z EnumMap türetmesi ve merkez overload'ının özel merkezi X/Y dönüşlerine aktarması ve Shapes.rotateHorizontal'ın dört yön EnumMap türetmesi ve Shapes.rotateAll'ın altı yön EnumMap türetmesi, rotateAll merkez overload'ının özel merkez aktarımı, Shapes.rotateAttachFace'in floor/wall/ceiling bileşik dönüşleri, Shapes.rotateHorizontal initial overload'ının, Shapes.rotateAll initial overload'ının ve Shapes.rotateAttachFace initial overload'ının başlangıç bileşikleri, merkez aktarımı ve geometri gözlemlerini belirliyor. | P01.d'de AABB contract/expandTowards(Vec3)/inflate/intersect/predicate overload'ları/collidedAlongVector/minmax/move/move overload'ları/sizes/hasNaN/positions/axis min/max/distanceToSqr/ofSize/setMin/setMax, AABB kutu kurucuları/üreticileri, koordinat ve iterable clip overload'ları, `BlockHitResult` sabit hit alanları, box, `Shapes.join` tarafından kabul edilen BooleanOp yolları ve tekli/çoklu shape collision, VoxelShape clip, closestPointTo, getFaceShape, Shapes.blockOccludes, mergedFaceOccludes, faceShapeOccludes ve Shapes.equal, Shapes.rotate, Shapes.rotateHorizontalAxis, Shapes.rotateAllAxis, Shapes.rotateHorizontal, Shapes.rotateAll ve Shapes.rotateAttachFace; world face/collider toplama ve reddedilen BooleanOp sabitleri ayrı kapsamdır. |
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
