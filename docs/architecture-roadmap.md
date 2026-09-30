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
- [ ] Varsayılan bileşenler tipli (`DataComponentMap`); `components.rs`
  ham bayt tutmaz.
- [ ] Büyü, tablo, enstrüman, müzik diski, zırh süsü ve bayrak deseni
  kayıtları `data/` JSON'undan ve sunucunun kayıt verisinden; `tooltip.rs`
  kalkar.
- [ ] Yaratıcı sekmeleri Java'daki gibi kodla kurulur; kayıt güdümlü
  parçalar (büyülü kitaplar, iksirler, tablolar, keçi boruları) oturumun
  kayıtlarından üretilir.
- [ ] Etkiler ve iksirler okunur tanımlar.

### F6. Bellek ve GPU

- [ ] Paletli section depolama (Java `PalettedContainer`).
- [ ] Java'nın blok vertex biçimi: renk ve ışık baytlara paketli, ~32 bayt.

### F7. UV kilidi

- [x] `uv_lock.rs` JOML ile açılışta hesaplanır (`invertAffine`, `mul`).

### F8. Sınıf başına şekiller

- [ ] `getShape` ve `getCollisionShape` blok sınıflarının kodundan;
  `outline.rs` ve `collision.rs` kalkar.

### F9. Ölçüp karar verilecek sapmalar

- [ ] D1: ışık kuyruğunun kare başına sınırı. Sapma: chunk ışığının
  açıldığı kare. Etki: büyük yüklemede son chunk'lar ~0,5 sn erken
  görünür; aynı anda çok chunk gelirse tek karede takılma riski. F4'ten
  sonra ölçülür, gecikme görülürse onayla yapılır.
- [ ] D2: chunk section'larını ağ thread'inde ayrıştırmak. Sonuç birebir
  aynı, yalnızca iş başka thread'de; giriş ve ışınlanmada ana thread
  takılması azalır.
