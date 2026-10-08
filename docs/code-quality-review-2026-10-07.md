# Kod kalitesi ve yapı incelemesi (2026-10-07)

Bu rapor depodaki belgelerden ve ajan yönlendirme dosyalarından **bağımsız** olarak yazıldı: ölçüt yalnız koddur ve genel Rust, oyun istemcisi ve Vulkan pratikleridir. Proje kurallarının kendisi de değerlendirmenin konusudur; çeliştikleri yer [§8](#8-proje-kurallarıyla-çelişen-öneriler)'de listelenir. Uygulanabilir adımlar görev panosuna Q/V/A/D görevleri olarak eklendi ([§9](#9-yol-haritası-ve-görevler)).

## 1. Özet

Kod davranış eşitliği açısından titiz (bit eşitliği, işlem sırası, kapsamlı oracle karşılaştırması), fakat **yapı açısından prosedürel ve düz** büyümüş: birkaç tanrı nesnesi (`ClientEntity`, `ClientWorld`, `PlaySession`, `VulkanBackend`) her yeni dilimle bir alan ve bir fonksiyon daha almış; tür hiyerarşileri tip sistemiyle değil `Option` alanlarıyla modellenmiş; çizim mantığı uygulama crate'ine sızmış. Vulkan arka ucu doğru çalışıyor ama CPU ve GPU'yu **her karede sıraya diziyor** ve FPS'i ekran tazeleme hızına kilitliyor. En büyük kazanımlar sırasıyla:

1. **Ölç** (bench + GPU zaman damgası), sonra 2 kare uçuşta ve fence beklemelerinin kaldırılması — tek başına en büyük FPS kazanımı beklenir.
2. **Tip sistemiyle modelle:** `EntityKind` enum'u, float/kimlik newtype'ları, tipli paket enum'u.
3. **Extract → render state → submit** ile entity çizimini tek geçişe indir, uygulama crate'ini incelt.
4. **Üretilmiş tabloları** Java'nın düzenini izleyen elle yazılmış modüllere çevir; JSON tabanlı veriyi çalışma anında oku.
5. **Public hijyen:** LICENSE, CI, İngilizce README, `.DS_Store` temizliği, log altyapısı.

## 2. Ölçümler

| Ölçü | Değer | Yorum |
| --- | --- | --- |
| Rust satırı (crates/) | ~221 bin | 28 bini üretilmiş tablo |
| En büyük dosyalar | `behavior_adapter/main.rs` 27k, `photon-world/src/tests.rs` 7k, `mob_model.rs` 5.5k, `vulkan.rs` 5.4k, `photon-world/src/lib.rs` 5k, `protocol.rs` 4.9k | 1–2k üstü dosya gezinmeyi ve incelemeyi zorlaştırır |
| `pub trait` sayısı | 12 | 221k satır için çok az soyutlama |
| `ClientEntity` alanı | ~110, 18'i `Option<Box<…State>>`, ~12'si `Option<…State>` | Kapalı tür hiyerarşisi düz yapıda |
| `ClientWorld` `pub fn` | 184 (tek `impl`) | Tanrı nesnesi |
| `*_bits: u32/u64` alanları | 103 | Float'lar `Eq` için bit olarak saklanıyor |
| Uygulamada `entity_type::id("minecraft:…")` | 33 | Çizim yolunda dize araması |
| `match packet_id` kolu (play) | 85; 18'i hatayı `if let Ok` ile yutuyor | Sessiz hata |
| `println!`/`eprintln!` | 83 | Log altyapısı yok |
| `.clone()` | 512 | Sıcak yollar gözden geçirilmeli |
| `BTreeMap` / `HashMap` | 139 / 102 | Chunk ve entity için sıralı ağaç |
| `vulkan.rs` içinde `SAFETY` yorumu | 0 (77 `unsafe`) | |
| Repo | LICENSE yok, CI yok, `rust-toolchain.toml` yok, `.DS_Store` izleniyor, 663 belge / 5.3 MB | Public görünüm |

## 3. Bulgular — modelleme ve tip güvenliği

### 3.1 `ClientEntity`: tür hiyerarşisi `Option` alanlarıyla
`crates/photon-world/src/entity.rs:661` her entity türünün durumunu aynı yapıda tutar (`rabbit`, `warden`, `panda`, `horse`, `minecart`… hepsi `Option`). Bir tavşanın `warden` alanı vardır; geçersiz durumlar temsil edilebilir, her tür eklemesi bu yapıya dokunur ve bellek düzeni şişer.

**Öneri — toplam tip (enum dispatch):** Java'nın kapalı sınıf hiyerarşisi Rust'ta en doğal hâliyle enum'dur.

```rust
pub struct Entity { pub base: EntityBase, pub living: Option<LivingState>, pub kind: EntityKind }
pub enum EntityKind { Rabbit(RabbitState), Warden(Box<WardenState>), Minecart(MinecartState), Other }
```
Ortak alanlar (`EntityBase`: konum, dönüş, hareket, bayraklar) ve `LivingEntity` katmanı ayrı yapılar; tür davranışı `match self.kind` ile. Trait nesnesi (`Box<dyn EntityBehaviour>`) yerine enum önerilir: tür kümesi kapalı, eşitlik testleri için `PartialEq` türetilebilir, sanal çağrı ve heap dağınıklığı olmaz.

### 3.2 Float'ların bit olarak saklanması
103 alan (`x_bits`, `y_rot_bits`, `swim_amount_bits`…) `f64::to_bits` ile saklanıyor; amaç `#[derive(Eq)]` ve bit eşitliği. Her okuma/yazma `from_bits`/`to_bits` gürültüsü taşır ve birim bilgisi kaybolur.

**Öneri — newtype + değer tipleri:**
```rust
#[derive(Clone, Copy, Debug)] pub struct JavaF64(pub f64); // Eq/Hash = to_bits eşitliği
pub struct DVec3 { pub x: JavaF64, pub y: JavaF64, pub z: JavaF64 }
```
Böylece `position`, `old_position`, `motion`, `lerp_target` üçlüleri tek alana iner; bit eşitliği tipin sözleşmesi olur.

### 3.3 Ham kimlikler
Entity kimliği, tür kimliği, blok durumu, menü kimliği hep `i32`. `OpenContainerMenu` (`lib.rs:184`) beş elemanlı isimsiz tuple'dır. Uygulama kodu türü her karede dizeyle arar (`foxes.rs`: `entity_type::id("minecraft:fox")`).

**Öneri:** `EntityId(i32)`, `BlockStateId(u32)`, `EntityType` (enum veya `&'static EntityTypeDef`) newtype'ları; tuple yerine adlandırılmış yapı. Karıştırılan kimlikler derleme hatası olur, dize araması kalkar.

### 3.4 Tanrı nesneleri
- `ClientWorld` (`photon-world/src/lib.rs:120`): chunk'lar, ışık, entity'ler, haritalar, tarif kitabı, taş kesici tarifleri, dünya sınırı, hava, binek kontrolü, tekne kürek paketleri, blok tahminleri — 184 public fonksiyonla tek `impl`.
- `PlaySession` (`photon-net/src/connection/play.rs:71`): soket, oyuncu durumu, ses kuyruğu, HUD, ölüm ekranı, sohbet indeksi ve **test izleme nesnesi** (`trace`) aynı yapıda; dünya `self.trace.world` içinde yaşıyor (108 kullanım). `*Observation` yapıları (`protocol.rs:598+`) üretim modülünde.
- `VulkanBackend` (`vulkan.rs:1285`): bkz. §5.

**Öneri — bileşim:** `ClientWorld { chunks: Chunks, lighting: Lighting, entities: Entities, weather: Weather, recipes: Recipes, border: WorldBorder, … }`; her alt sistem kendi modülü ve `impl`'iyle. `PlaySession` → `Connection` (I/O) + `ClientState` (oyun) + test için ayrı `Recorder` (özellik bayrağı veya test crate'i).

## 4. Bulgular — katmanlar ve akış

### 4.1 Çizim mantığı uygulama crate'inde, tür başına tarama
`crates/photon/src` 69 modül; `foxes.rs`, `pandas.rs`, `sheep.rs`… her biri `world.entities().filter(|e| e.type_id == fox)` ile **tüm entity'leri** tarar ve yeni `Vec<Vertex>` döner. `mobs.rs:600-660` bu fonksiyonları ardışık çağırır: kare başına O(tür sayısı × entity sayısı) tarama ve onlarca ayrı ayırma. Uygulama crate'i hem `photon-net::PlaySession`'ı hem render modellerini bilir.

**Öneri — extract → render state → submit:** Java'nın kendisi 1.21.2'den beri bu modeli kullanır (`EntityRenderer.extractRenderState` → `submit`); Bevy'nin "extract" fazı da aynı fikirdir.
1. *Extract:* dünya tek geçişte gezilir, her entity için küçük, salt-okunur bir render durumu (`EntityRenderState` + türe özgü kısım) üretilir.
2. *Dispatch:* tür → çizici tablosu (`EntityRenderDispatcher` karşılığı; `match` veya `static` dizi) tek döngüde çağrılır.
3. *Submit:* çiziciler önceden ayrılmış, kare başına yeniden kullanılan `RenderType` kovalarına yazar (`Vec::clear`, yeniden ayırma yok).
Çizim modülleri `photon-render::entity::*` altına taşınır; uygulama yalnız döngü, pencere ve girdi tutar. Extract adımı ağdan bağımsız olduğu için ayrıca test edilebilir ve ileride paralelleştirilebilir.

### 4.2 Paket işleme
`play.rs:1563` `handle` 85 kollu bir `match packet_id`'dir; ayrıştırma, durum güncellemesi ve iz kaydı iç içedir; 18 kol ayrıştırma hatasını `if let Ok` ile iz bırakmadan atar.

**Öneri — functional core / imperative shell:**
```rust
pub enum ClientboundPlay { Login(Login), AddEntity(AddEntity), SetEntityMotion(SetEntityMotion), /* … */ }
pub trait Decode: Sized { fn decode(buf: &mut Reader) -> Result<Self, ReadError>; }
pub trait PlayListener { fn handle(&mut self, packet: ClientboundPlay) -> Result<(), SessionError>; }
```
Ayrıştırma saf ve tek yerde (fuzz/test edilebilir), uygulama `PlayListener` (Java `ClientPacketListener`) içinde; okunamayan paket `log::debug!` ile kaydedilir. Okuyucu iş parçacığı zaten var (`play.rs:168`); kanal ham bayt yerine çözülmüş `ClientboundPlay` taşıyabilir, böylece çözme ana döngüden çıkar.

### 4.3 Lint'i aşmak için tuple parametreler
`too_many_arguments` sınırı, parametreleri anlamsız tuple'lara paketleyerek aşılmış: `(view, &mut held)`, `(models, armor)`, `(&mut self.armor, Worn { … }, &self.entity_textures)` (`play.rs:675`, `mobs.rs:600+`). Lint'in amacı (bağlamı adlandırmak) gerçekleşmemiş, okunurluk düşmüş.

**Öneri:** adlandırılmış bağlam yapıları (`FrameContext { view, partial, textures, models }`, `HeldItems`), ya da §4.1'deki dispatcher'ın tek `RenderContext` parametresi.

### 4.4 Gözlemlenebilirlik
83 `println!`/`eprintln!`; seviye, filtre ve hedef yok. **Öneri:** `log` + basit bir arka uç (veya `tracing`), betik çıktıları için ayrı `--report` biçimi; profil için `tracing` span'ları (kare, tik, mesh, yükleme).

### 4.5 Veri yapıları
Chunk'lar (`BTreeMap<(i32,i32), ClientChunk>`) ve entity'ler (`BTreeMap<i32, ClientEntity>`) sıralı ağaçta. Sık erişilen, rastgele anahtarlı haritalar için hash tablosu (sabit hasher) veya slab/`slotmap` daha uygundur; Java'nın gözlemlenebilir yineleme sırası gerekiyorsa ayrı bir sıra listesi açıkça tutulmalı. Değişiklik ancak §5.6 ölçümüyle yapılır.

## 5. Bulgular — Vulkan ve FPS

`crates/photon-render/src/vulkan.rs` 5.4k satırlık tek dosya; instance, cihaz, swapchain, bellek, 20+ boru hattı, doku, kayıt ve yok etme birlikte.

| # | Bulgu | Kanıt | Etki |
| --- | --- | --- | --- |
| 5.1 | **Tek kare uçuşta:** tek komut tamponu, tek `in_flight` fence; `render_frame` başında beklenir | `vulkan.rs:4961`, alanlar `:1300`, `:1396` | CPU ve GPU sırayla çalışır; ikisinin toplam süresi kare süresi olur |
| 5.2 | **Her `set_*` yüklemesi fence bekler** | `fill_bytes` `:4695`, `set_weather` `:4735` | Kare içinde ~20 senkron bekleme |
| 5.3 | Her yüklemede `map_memory`/`unmap_memory`; tampon başına `vkAllocateMemory` (HOST_VISIBLE\|COHERENT) | `:4718`, `upload_buffer :1709` | Sürücü çağrısı yükü; ayırma sayısı sınırı |
| 5.4 | ~20 `Option<SkyBuffer>` + `*_count` alan çifti; `record_scene` içinde elle tekrarlanan bind/push/draw blokları | `:1285-1398`, `:2078-2620` | Her yeni çizim türü 3 yerde kod; hata yüzeyi |
| 5.5 | Yalnız `PresentModeKHR::FIFO` | `:1546`, `:3242` | FPS ekran hızına kilitli; vsync/kare sınırı seçeneği yok |
| 5.6 | Ölçüm yok | — | Optimizasyonlar kanıtlanamaz |
| 5.7 | GLSL Rust string sabitleri, açılışta `shaderc` ile derleme; `VkPipelineCache` yok | `:95-905`, `:2657` | Açılış süresi, C++ derleme bağımlılığı, editör/araç desteği yok |
| 5.8 | Non-indexed çizim, 48 baytlık `Vertex` (12 float) | `scene.rs:25` | Quad başına 6 köşe; bellek bant genişliği |
| 5.9 | Bölüm başına ayrı draw + push constant | `bind_section :1849` | Uzak görüş mesafesinde CPU darboğazı |
| 5.10 | Renk temizliği `cmd_clear_color_image` + ek düzen geçişleri, sonra `LOAD` | `render_frame :4954` | Gereksiz bariyer; dynamic rendering `CLEAR` yeterli |
| 5.11 | RAII yok, tek büyük `Drop`; `SAFETY` yorumu yok | `:5145` | Sızıntı/çift serbest bırakma riski, inceleme zorluğu |

**Önerilen mimari:**
1. **Modüller ve sahiplik:** `vk/{instance,device,swapchain,memory,pipeline,texture,frame}.rs`; her Vulkan nesnesi sahibi olan küçük bir tipte (`Buffer`, `Image`, `Pipeline`), yok etme sırası tipte belgeli. Alt ayırıcı olarak mevcut arena genelleştirilir veya `gpu-allocator` (saf Rust, VMA benzeri) değerlendirilir.
2. **Kare başına kaynak (2 kare uçuşta):** `FrameResources { cmd, fence, image_available, ring: RingBuffer }`; `set_*` fonksiyonları fence beklemez, o karenin kalıcı map'li halka tamponuna yazar. Tamponu GPU'nun kullandığı kareye dokunulmaz (Khronos "frames in flight").
3. **Veri olarak çizim türleri:** `RenderType { pipeline, blend, depth_write, descriptor, buffer_slot }` tablosu (Java `RenderPipelines`/`RenderTypes` ile aynı adlar); kare, `(RenderType, first, count)` listesi olarak kaydedilir ve tek döngüyle çizilir. 20 alan çifti tek `DrawLists` yapısına iner.
4. **Present modu ve kare sınırı:** Java'nın `enableVsync` ve `framerateLimit` seçeneklerine bağlı `MAILBOX`/`IMMEDIATE`/`FIFO` seçimi ve CPU tarafı kare sınırlayıcı.
5. **Shader'lar dosyada:** `crates/photon-render/shaders/*.{vert,frag}`, `build.rs` ile SPIR-V'ye derlenip `include_bytes!`; diskte `VkPipelineCache`.
6. **Geometri:** paylaşılan quad indeks tamponu (köşe sayısı 6→4), Java'nın vertex formatlarıyla hizalı kompakt köşe (konum + paketli renk/UV/ışık; Sodium bu yolla bellek ve bant genişliğini ~%40 azaltır).
7. **Bölümler:** bölüm ofsetleri SSBO'da, `vkCmdDrawIndirect`/multi-draw ile katman başına tek çağrı (Sodium: dünya çiziminde CPU yükü ~%90 düşer).
8. **Ölçüm önce:** GPU zaman damgası sorguları (pass başına), kare süresi p50/p99, sabit betikli sahne ile `bench` komutu; her optimizasyon öncesi/sonrası sayı ve `capture_frame` ile piksel eşitliği.

## 6. Bulgular — üretilmiş veri

`crates/photon-data/src/generated/` 28 dosya / 28k satır (`animations.rs` 8.1k, `entity_models.rs` 4.3k, `creative_tabs.rs` 4.3k, `blocks.rs` 2.4k, `sound_events.rs` 2k…) ve 1.5k satırlık Python üretici (`tools/generate/`). Tablolar tek dev `static` dizi; Java'da ise her veri bir sınıfa veya JSON dosyasına aittir (`WardenAnimation`, `PigModel.createBodyLayer`, `Blocks`, `SoundEvents`). Diff'ler okunur ama gezinme ve sahiplik zayıf; sürüm atlamasında üretici zinciri (Java oracle + Python + yeniden üretim) bakım yüküdür.

**Öneri — üç sınıf:**
- **(a) Java kodunda tanımlı** (bloklar, eşyalar, ses olayları, entity türleri, öznitelikler, model katmanları, animasyonlar): Java'nın dosya düzenini izleyen **elle yazılmış** Rust modülleri — `photon-data/src/model/pig.rs::create_body_layer()`, `animation/warden.rs`, `block/blocks.rs` içinde Java'daki sırayla tanımlar. Okunur bir oluşturucu DSL'i (`Block::of(..).strength(..)`) zaten var; korunur.
- **(b) JAR `data/` JSON'u veya sunucunun eşitlediği kayıtlar** (tablo/kurt varyantları, harita renkleri vb.): kaynak koda girmez; çalışma anında kullanıcının yerel oyun dosyasından veya `RegistryData` paketinden okunur (assetlerle aynı yol).
- **(c) Java'nın hesapladığı:** hesaplanır.

Oracle karşılaştırması **doğrulama testi** olarak kalır (tabloyu üretmez, elle yazılmışı denetler). Tek seferlik dönüşüm mevcut çıktıdan betikle yapılabilir; ardından `tools/generate/` ve `generated/` silinir ve dosyalar elle bakılır.

**Hukuki not:** Java tanımlarının elle aktarılması da Mojang'ın koduna dayanan türev veridir; Azalea, Valence ve Pumpkin benzer veriyi depoda tutar ve Mojang bunlara karşı işlem yapmamıştır, ama bu bir güvence değildir. Proje adı resmî görünmemeli ve ücretsiz kalmalıdır (Minecraft Usage Guidelines).

## 7. Bulgular — depo ve public görünüm

- **LICENSE yok** (Rust ekosisteminde yaygın: MIT/Apache-2.0 ikili lisans) — karar sahibinindir.
- **CI yok:** `.github/workflows` altında fmt, clippy, build ve referans gerektirmeyen testler; `cargo-deny` (lisans, güvenlik tavsiyeleri, yinelenen bağımlılık).
- `rust-toolchain.toml` yok (edition 2024 ve `as_chunks_mut` gibi yeni API'ler kullanılıyor; sürüm sabitlenmeli).
- `.DS_Store` Git'te izleniyor.
- README Türkçe, "hazırlık deposu" diyor ve güncel değil; CONTRIBUTING/SECURITY yok. Public hedef kitle için İngilizce README (ne, neden, ekran görüntüsü, kurulum, durum), İngilizce kod yorumları zaten var.
- 663 belge (5.3 MB) çoğunlukla dilim kayıtları; kökte kodu gölgeliyor. Öneri: `docs/` (mimari, katkı, test) ve `docs/records/` (dilim/parity kayıtları) ayrımı.
- Commit başlıkları 200+ karakter; öneri: ≤72 karakter başlık + gövde (Conventional Commits zaten kullanılıyor).
- Test harness'ı (`crates/photon/examples/behavior_adapter`, 27k+ satır) uygulama crate'inin `examples/`'ında; ayrı bir `crates/photon-oracle` (publish=false) crate'i daha anlaşılırdır. `photon-world/src/tests.rs` (7k) konuya göre bölünmeli.

## 8. Proje kurallarıyla çelişen öneriler

- "Tablo oracle'dan yakalanır, üretilmiş Rust dosyasına yazılır" kuralı §6 ile çelişir; kullanıcı kararıyla D07'de kural metinleri güncellenecek.
- Karmaşıklık sınırları (`too_many_arguments` 7) tuple paketlemeyle aşılıyor (§4.3); sınır doğru, uygulama biçimi değil — bağlam yapısı zorunlu tutulmalı.
- "Şimdilik engine, renderer, ECS, ses veya ağ framework'ü ekleme" kuralı doğru; bu rapor da framework önermez. Önerilen bağımlılıklar küçük ve yerine konabilir: `log`, (isteğe bağlı) `gpu-allocator`, `cargo-deny` (yalnız CI).

## 9. Yol haritası ve görevler

Etki/maliyet sırasıyla; ayrıntı ve kabul ölçütleri görev dosyalarında ([pano](tasks/README.md), "Kalite ve performans" satırları).

| Akış | Görevler | Neden önce |
| --- | --- | --- |
| Public hijyen | Q01 → Q02 → Q03 | Ucuz, görünür, CI sonraki her görevi korur |
| Vulkan/FPS | V00 → V01 → V02, V03 → V04, V05 → V06 | Ölçmeden optimizasyon yok; bölmeden yeniden yapılandırma riskli |
| Modelleme | A01 → A02 → A03; A04; A05; A06 | Newtype'lar sonraki taşımaları ucuzlatır |
| Veri | D01 → D02…D06 → D07 | Envanter sınıflandırmayı sabitler |

## 10. Benzer projeler

| Proje | Dil / tür | Alınacak ders |
| --- | --- | --- |
| Azalea | Rust, bot/istemci kitaplığı | Crate ayrımı (protocol, world, physics, registry, block); paketler tipli enum; veri üreticiden (artık Pumpkin extractor) |
| Valence | Rust, sunucu çatısı | Fabric extractor → JSON → `build.rs`; tipli paketler ve `Decode`/`Encode` trait'leri |
| Pumpkin | Rust, sunucu | Extractor JSON'u; geniş topluluk, CI ve katkı belgeleri örneği |
| Stevenarella / Leafish | Rust, istemci (OpenGL) | Blokların **elle** yazılmış makroyla tanımlanması — §6(a)'ya en yakın model |
| Sodium | Java, render modu | Kompakt vertex, multi-draw, erken yüz ayıklama |
| Bevy | Rust, motor | Extract → render ayrımı, kare başına kaynaklar |

## 11. Açık kararlar

1. Lisans (MIT/Apache-2.0 ikili önerilir).
2. Varsayılan present modu (Java varsayılanı vsync açık; kare sınırı seçeneği).
3. JSON tabanlı verilerin çalışma anında hangi kaynaktan okunacağı (yerel oyun JAR'ı vs. sunucu kayıtları) — D01 envanterinde tablo tablo.
4. `gpu-allocator` bağımlılığı mı, mevcut arenanın genelleştirilmesi mi (V03'te ölçümle).

## Kaynaklar

- [Azalea codegen](https://git.matdoes.dev/mat/azalea/src/commit/54e00eb3df12262a0b31c11600acedf1544ac4fb/codegen), [azalea-registry](https://docs.rs/crate/azalea-registry/^0.4.0)
- [Valence extractor](https://tangled.org/avikav.net/valence/tree/main/extractor)
- [Sodium 0.3.0 sürüm notları](https://github.com/CaffeineMC/sodium-fabric/releases/tag/mc1.17.1-0.3.0)
- [Vulkan Tutorial: Frames in flight](https://docs.vulkan.org/tutorial/latest/03_Drawing_a_triangle/03_Drawing/03_Frames_in_flight.html)
- [Arm: Vulkan descriptor and buffer management](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/vulkan-descriptor-and-buffer-management)
- [Minecraft EULA](https://minecraft.net/en-us/eula)
- Rust API Guidelines (rust-lang.github.io/api-guidelines), Clippy lint listesi, `cargo-deny` belgeleri
