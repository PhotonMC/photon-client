# Blok modeli, tablolar ve performans yol haritası (2026-09-30)

Bu belge veri tablolarının ve performansın Java'nın yaptığı gibi yeniden
kurulmasını adım adım izler. İlke: Java bir değeri başka veriden
**hesaplıyorsa** biz de hesaplarız; Java onu kodda veya `data/` altında
**tanımlıyorsa** biz de tanımlı, tipli ve okunur bir kayıt tutarız.
Tekrarları silmek için verilmiş indeksler, bit dizgeleri, ham ağ baytları
ve koşu uzunluğu dizileri kalmaz. Her adım parity ile doğrulanır; bilinçli
bir sapma ancak etkisi yazılıp onaylandıktan sonra yapılır.

## Başlangıç ölçümleri

Oyun açılmadan, `crates/voxel/examples/bench.rs` ile (8 çekirdek, release):

| Ölçü | Değer |
| --- | --- |
| Blok modellerinin açılışta hazırlanması | ~660 ms (disk önbellekte), ~1,45 sn soğuk |
| Model çözümleme | ~310 ms; 3.407 dosya için 8.046 okuma |
| Section mesh'i (1.000–2.250 quad) | 2,0–3,1 ms |
| Section bölge kopyası | 0,065 ms |

Meshing profilinin tepesi: `block_index_for_state` (ikili arama),
`should_render_face` (98 çift içinde doğrusal arama, dizge ile özellik
okuma), blok başına dizge üretimi ve `BTreeMap<String>` araması, blok
başına model dönüşü ve köşe hesabı.

## Fazlar

### F0. Ölçüm aracı

- [x] `bench` örneği: açılış aşamaları ve yapay bir dünyada section
  meshing süresi; her performans adımı bununla ölçülür.

### F1. Blok durumu önbelleği (Java `BlockStateBase.Cache`)

Her state için açılışta bir kez hesaplanan, state numarasıyla doğrudan
erişilen bir kayıt. Hesaplanan değerler Java'nın formülüyle çıkar; tablo
olarak tutulmaz.

- [x] State → blok ve sıra: tek dizi, ikili arama yok.
- [x] Yüz oklüzyon şekilleri (`getOcclusionShape`, `getFaceShape`) ve
  `shouldRenderFace`; `skipRendering` sınıfların kendi kuralıyla.
  `face_occlusion.rs` silinir.
- [x] Sağlam yüzler ve hareket engelleme (`isFaceSturdy`, `legacySolid`,
  `blocksMotion`); `fluid_flow.rs` ve `motion_blocking.rs` silinir.
- [x] Işık özellikleri: `solidRender`, `propagatesSkylightDown`, sönüm,
  gölge parlaklığı, çarpışma şeklinin tam blok olması; `light.rs`'in
  hesaplanan alanları kalkar.
- [x] Akışkan durumu (`getFluidState`) sınıf ve özelliklerden;
  `fluid_overlay.rs` sınıftan (`HalfTransparentBlock`, `LeavesBlock`).

### F2. Okunur blok tanımları (Java `Blocks.java`)

- [x] Blok başına tek kayıt: ad, sınıf (enum), özellikler ve tanımlı
  `Properties` alanları (kırılma süresi, sürtünme, hız ve zıplama çarpanı,
  ışık yayma, `noOcclusion`, ofset türü, yüzey rengi…), alan adlarıyla.
  `states`, `destroy`, `light`, `movement`, `offset`, `behaviour`
  tabloları bunda birleşir.
- [x] Sınıf davranışları sınıf enum'u üzerinden; `ClientBlockStateRegistry`
  içindeki sabit merdiven numaraları kalkar, bütün merdivenler
  özelliklerinden çalışır.
- [x] Etiket bayrakları (tırmanılabilir, çit, duvar…) sunucunun gönderdiği
  etiketlerden; gömülü etiket listesi kalkar.

### F3. Hazır modeller ve paralel açılış (Java `ModelBakery`)

- [x] Ayrıştırılmış model önbelleği: her dosya bir kez okunur.
- [x] Her state'in modeli açılışta hazırlanır (dönüş, UV kilidi, köşeler);
  mesh sırasında yalnızca varyant seçimi, renk ve ışık kalır; blok başına
  dizge işi kalmaz.
- [x] Rayon ile paralel dosya okuma, PNG çözme, mipmap ve state hazırlama.

### F4. Görünürlük ve derleme zamanı (Java `SectionOcclusionGraph`)

- [x] Frustum culling: çizim yalnızca görüş konisindeki section'lar için.
- [x] Oklüzyon grafiği: section'lar arası görünürlük; görünmeyen section
  ne derlenir ne çizilir.
- [x] Derleme 8 komşu chunk ve ışık hazır olunca (`hasAllNeighbors`,
  `lightOnInColumn`); `SETTLE` süresi kalkar, çift derleme biter.

### F5. Eşyalar ve veri güdümlü kayıtlar

- [x] Eşya tanımları `Items.java` gibi: `Item::of(ad, sınıf)` zinciri,
  sınıf hiyerarşisi (`ItemClass`), yığın boyutu, dayanıklılık, alet,
  giyilebilirlik, bırakılan kalıntı; `stacking.rs`, `behaviour.rs` ve alet
  tablosu kalktı.
- [x] Varsayılan bileşenler tipli (`Component` enum'u, eşya tanımında);
  `components.rs` ham baytı kalktı, varsayılanlar ağ biçimine açılışta
  kodlanıyor (`default_components.rs`) ve parity'de Java'nın bayt ve
  JSON'uyla karşılaştırılıyor. Tüm bileşen türlerinin okuma sınaması
  `tests/behavior/fixtures/item-default-patches.txt` fixture'ında kaldı.
- [x] Büyü, tablo, enstrüman, müzik diski, zırh süsü ve bayrak deseni
  kayıtları `data/` JSON'undan ve sunucunun kayıt verisinden; `tooltip.rs`
  kalkar.
- [x] Yaratıcı sekmeleri bildirim olarak: her yığın eşya adı ve bileşenleri (`CreativeStack::of(..).with(..)`); kayıt güdümlü
  parçalar (büyülü kitaplar, iksirler, tablolar, keçi boruları) oturumun
  kayıtlarından üretilir.
- [x] Etkiler ve iksirler okunur tanımlar.

### F6. Bellek ve GPU

- [x] Paletli section depolama (Java `PalettedContainer`).
- [ ] Java'nın blok vertex biçimi: renk ve ışık baytlara paketli, ~32 bayt.

### F7. UV kilidi

- [x] `uv_lock.rs` JOML ile açılışta hesaplanır (`invertAffine`, `mul`).

### F8. Sınıf başına şekiller

- [x] `getShape` ve `getCollisionShape` blok sınıflarının kodundan
  (`voxel-world/src/block/shape_rules/`); `outline.rs`, `collision.rs` ve
  üreticisi kalktı. Oracle vakaları hesaplanan durum şekillerinden üretilir.

### F9. Ölçüp karar verilecek sapmalar

- [ ] D1: ışık kuyruğunun kare başına sınırı. Sapma: chunk ışığının
  açıldığı kare. Etki: büyük yüklemede son chunk'lar ~0,5 sn erken
  görünür; aynı anda çok chunk gelirse tek karede takılma riski. F4'ten
  sonra ölçülür, gecikme görülürse onayla yapılır.
- [x] D2: chunk section'larını ağ thread'inde ayrıştırmak. Sonuç birebir
  aynı, yalnızca iş başka thread'de; giriş ve ışınlanmada ana thread
  takılması azalır.
- [x] Durum önbelleğinin şekilleri rayon ile paralel kurulur (200 → ~105 ms)
  ve bölüm mesh işçileri rayon havuzunda çalışır (elle yazılmış
  `Mutex<Receiver>` havuzu kalktı).

## Son ölçümler (bench, 8 çekirdek, release)

| Ölçü | Başlangıç | Şimdi |
| --- | --- | --- |
| Blok modellerinin hazırlanması | ~660 ms | ~380 ms |
| Durum önbelleği (şekiller dahil) | — | ~105 ms |
| Section mesh'i (kısa / yoğun) | 2,0–3,1 ms | 0,6 / 4,0 ms |
| Section blok durumları (yapay dünya) | 9,8 MB | 0,3 MB |

Mesh özetleri (`digest=`) bütün adımlarda aynı kaldı. Açık kalanlar: F6.2
(vertex biçimi; görsel doğrulama gerektirir) ve D1 (sapma; onay gerekir).

## Bellek ve yükleme turu (2026-09-30)

- [x] `voxel-data` üretilen dosyaları `src/generated/` altında; yollar aynı.
- [x] CPU atlas kopyaları (3 × ~43 MiB), ekran görüntüsü okuma tamponu ve GUI
  atlas görüntüsü artık tutulmuyor; arena ve staging boş blokları sürücüye
  geri veriyor. Oyun toplam ayak izi (`footprint`, 854×480, girişten 30 sn
  sonra): 643 MB → ~531 MB. Kalan: GPU ~220 MB (atlas, section arenası),
  Rust yığını ~70 MB canlı, gerisi başlangıç tepesinden kalan serbest
  sayfalar (`malloc_zone_pressure_relief` bunları geri vermedi).
- [x] Section boşluk denetimi her karede 10.000 bekleyen section'ı taramıyordu
  artık yalnız kirlenenleri tarıyor (`sections.rs`): yük sonrası kare süresi
  ~16,7 ms → ~8,4 ms (ana thread CPU'su).
- Chunk yükleme ölçümü (bench): 25 chunk `replace_chunk` ~9 ms (chunk başına
  ~0,4 ms), section bölge kopyası ~80 µs; işçiler yük sırasında %99 boşta.
  Yani chunk uygulamayı rayon'a taşımak anlamlı kazanç vermez; yavaşlık
  sunucudan gelen veri, ışık kuyruğu (D1) ve görünürlükle sınırlı.
