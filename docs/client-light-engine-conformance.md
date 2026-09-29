# P08.c-light.2 — İstemci ışık motoru ve yeniden aydınlatma — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity`

## Neden

Vanilla sunucu ışık değişikliği paketini yalnız görüş mesafesinin kenarındaki
oyunculara gönderir (`ChunkHolder.broadcastChanges`, `getPlayers(pos, true)`).
Görüşün içinde istemci ışığı kendisi hesaplar: `ClientLevel`'ın
`LevelLightEngine`'i. Bu motor yokken kırılan bir bloğun yeri ve altı
karanlık kalıyordu.

## Ne pinlendi

`crates/voxel-world/src/light.rs`, referansın ışık paketinin birebir portu:

- `DataLayer` ve katman paylaşımı: sunucudan kuyruğa alınan katman, bölüm
  depolanınca aynı nesne olarak güncel haritaya girer; yayılım o nesneyi
  değiştirir ve "aynı katman mı" karşılaştırması kimlikle yapılır. Katmanlar
  bir arenada tutulur, haritalar kimlik taşır.
- `LayerLightSectionStorage`: bölüm durumları (veri biti ve 26 komşu sayısı),
  bölüm kurma/kaldırma, kuyruk ve tutma, her çalıştırmada kopyada yazma ve
  görünür kopyanın yayını; `SkyLightSectionStorage`'ın sütun tepe bölümleri
  ve en alçak bölüm, üstteki katmanın ilk satırını tekrarlayan yeni katman,
  veri olmayan bölümde üstteki ilk katmanın alt satırı.
- `BlockLightEngine` ve `SkyLightEngine`: blok kontrolü, azalma ve artış
  kuyrukları, opaklık, yüz şekli örtmesi, ışık yayan blok, gök kaynaklarının
  sütunda kaldırılıp eklenmesi, boş bölümlerin altından yayılım, ışığı açma
  ve kaynaklardan yayma.
- `ChunkSkyLightSources`: sütun başına gökyüzünün dikey indiği en alçak y,
  yığından kurulum ve blok değişiminde güncelleme.
- İstemci bağlantısı: yığın ışığı ve ışık paketi `queueLightUpdate` gibi
  kuyruğa girer, kare başına `pollLightUpdates` bütçesiyle (onda biri, en az
  on, binden sonra hepsi) işlenip motor çalışır; `enableChunkLight`,
  `readSectionList` yeniden çizim işaretleri, `queueLightRemoval` (yığın
  yüklü olmasa da) ve blok değişiminde `LevelChunk.setBlockState`'in ışık
  kancaları (bölüm boşluğu, gök kaynakları, `checkBlock`).

Işığın okuduğu iki yeni alan referanstan tabloya eklendi ve her koşuda
karşılaştırılıyor: durum başına `useShapeForLightOcclusion` ile
`canOcclude` (`block/light-properties`, `crates/voxel-data/src/block/light.rs`),
iki yüz şeklinin birleşiminin bütün yüzü örtüp örtmediği
(`Shapes.faceShapeOccludes`; `block/face-occlusion` içindeki `covers`,
`crates/voxel-data/src/block/face_occlusion.rs`).

## Senaryolar

`tools/behavior/cases.py` içindeki `movement/light-changes`. Oracle,
[movement fixture'ındaki](movement-fixture-record.md) gerçek `ClientLevel`'ın
ışık motorunu sunucu verisi olmadan, sunucunun ilk aydınlatmasındaki sırayla
kurar (dolu bölümlerin durumu, ışığı açma, kaynaklardan yayma, çalıştırma).
Sonra her olay istemcinin blok güncellemesi yolundan
(`setServerVerifiedBlockState`) gelir ve motor çalışır. Başlangıçta ve her
olaydan sonra bir kutunun blok ve gök ışığı karşılaştırılır.

Dallar: boşluğa kadar kazma (gök kaynağının sütunda inmesi, boş bölümlerin
altına yayılım) ve geri kapatma; kapalı odada ışık yayan blok koyup kaldırma,
tavanı delip kapatma; gökyüzü altında meşale, alt ve üst yarım blok,
yapraklar, su, cam, merdiven; yığın sınırında kazma ve ışık yayan blok.

## Sonuç

`compare --family block --family movement`: `parity_passed`. Math
ailesindeki ışık kuyruğu kayıtları
([P08.c-light.1](client-world-light-queue-parity-conformance.md) ve
kaldırma vakası) yeni motorla doğrudan çalıştırıldı ve eşleşti; math ailesinin
tamamı macOS'ta başka bir nedenle koşamıyor ([yerel kurulum](local-setup.md)).

Canlı: vanilla 26.2 sunucusunda aşağı kazılan çukurun dibi gök ışığıyla
aydınlanıyor; kare başına ışık işi 1 ms'yi aşmadı.

## Eşitlik olmayanlar ve açıklar

- Referansın blok kontrol kümesi ve kuyruk yinelemesi hash sırasıyla
  yürür; port ekleme sırasını kullanır. Işık düzeyleri sıradan bağımsız
  sabit noktaya ulaştığı için sahnelerde fark yok, ara durumlar farklı
  olabilir.
- Işık değişince bölüm yeniden çizimi etkilenen bölümle sınırlı; referans
  AO ve ton önbelleklerini de geçersiz kılar.
