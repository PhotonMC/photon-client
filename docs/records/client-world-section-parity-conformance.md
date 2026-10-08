# P08.a/b — LevelChunkSection storage ve ClientWorld lookup — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 chunk section payload'ının Java `LevelChunkSection`
tarafından okunmasını ve aynı section verisinin Rust `ClientWorld` içinde dünya
koordinatlarından sorgulanmasını karşılaştırır. Kanıt sınıfı `client-parity`'dir:
aynı özgün sentetik byte girdisi gerçek Java oracle'ına ve üretim Rust parser/world
yoluna verilir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynak incelemesinde şu yollar kullanıldı:

- `world/level/chunk/LevelChunkSection.java:12-47,57-83,122-186,193-195`,
  section sayaçlarını, blok state/biome container'larını, `read` sırasını ve
  yerel `getBlockState`/`getNoiseBiome` sorgularını tanımlar.
- `world/level/chunk/PalettedContainer.java:77-102,105-147,160-170,184-223`,
  wire bits değerinin yeni storage/palette ile okunmasını ve sabit long
  storage'ının bağlanmasını tanımlar.
- `world/level/chunk/Strategy.java:20-68`, blokta tek değerli, 4 bit linear,
  5–8 bit hash map ve daha geniş global palette dallarını; biyomda tek değerli,
  1–3 bit linear ve daha geniş global palette dallarını seçer.
- `util/SimpleBitStorage.java:244-264,294-301`, `64 / bits` değer-per-long
  hesabını ve düşük bitlerden başlayan hücre sırasını tanımlar.
- `network/protocol/game/ClientboundLevelChunkPacketData.java:27-66,99-110`
  section byte buffer'ının chunk packet içindeki yerini; `ClientPacketListener.java:888-891`
  ise decoded section'ların client chunk cache'e aktarımını gösterir.

## Davranış sözleşmesi

- `LevelChunkSection.read`, önce signed short non-empty/fluid sayaçlarını,
  sonra blok state container'ını, sonra biome container'ını okur. Sayaçlar read
  sırasında yeniden hesaplanmaz; section sorguları bu header state'ini korur.
- Blok section'larında wire bits `0` tek değerli palette, `1..4` 4 bit linear
  storage, `5..8` hash map palette ve daha geniş değerler global palette yoluna
  gider. Biyom section'larında karşılık gelen local aralık `1..3` bittir.
- Container index'i x'in en hızlı değiştiği `(y << 4 | z) << 4 | x` blok
  sırasını ve `(y << 2 | z) << 2 | x` biyom sırasını kullanır.
- Chunk payload'ındaki section sırası dimension `min_y / 16` section'ından
  başlar. Dünya sorgusu chunk ve section koordinatlarını floor division ile,
  yerel hücreleri Euclidean remainder ile çözer. Yüklenmemiş chunk veya section
  sorgusu sonuç üretmez.

## Yeni dal ve varyant kararı

Bu, mevcut yapısal Rust section parser testinden farklı olarak ilk kez gerçek
Java `LevelChunkSection.read` çağrısını, palette dallarını ve ardından
`ClientWorld` dünya koordinatı sorgusunu aynı kanıtta çalıştıran daldır. Önceki
`client-world-state-conformance.md` canlı vanilla kaydı `server-conformance`
sınıfındadır; wire kabulünü gösterir fakat Java istemci davranışıyla eşitlik
göstermez. Bu nedenle aynı sonucu tekrar eden ayrı sabit varyantlar açılmadı:
tek parametrik kayıt dört palette dalını, negatif chunk/boundary koordinatlarını
ve eksik sorguyu birlikte taşır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/section-read-and-coordinate-lookup` kaydı dört section payload'ı
üretir:

- tek değerli air/blok ve plains biyom section'ı,
- 4 bit blok ve 2 bit biyom linear palette'i,
- 5 bit blok hash map ve 3 bit biyom linear palette'i,
- 15 bit blok ve 7 bit biyom global palette'i.

Sorgular `chunk_x=-2`, `chunk_z=3`, `min_y=-64` ile section tabanını, word
sınırlarını, negatif koordinat dönüşümünü ve yüklenmemiş konumu gözlemler.
Java oracle gerçek `LevelChunkSection.read` ve public section getter'larını
çalıştırır; sayaç gözlemi için yalnızca fixture reflection'ı kullanılır.

Java referans kimliği:
`708bc940d38bdc65bc5a0b7eb60db78e2d42d335ca76457241d1a5191ebc7449`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2964** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-net/src/connection/protocol.rs` içindeki üretim section parser'ı
`decode_chunk_section_data` üzerinden adapter'a açılır. Adapter aynı decoded
section değerlerini `ClientWorld::replace_chunk` ile yazar; blok state,
biome, section counter ve missing lookup sonuçlarını
`block_state_at`, `biome_at` ve `section_counts_at` ile alır.

Debug adapter SHA-256 değeri
`0fd0c525a4665404af2adb2099b7d9209abcfa768a0a8f7f5e3d0a09a50d03c4` ile aynı
capture üzerinde **2964/2964** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`5820c8beef7af7e3b396c5c789779ae06a3b419646c47f44e30ff0712c0c8560` ile aynı
capture üzerinde **2964/2964** eşleşti; fark `null` kaldı.

## Vanilla server-conformance ayrımı

Bu dilim yeni serverbound paket üretmedi. Orijinal Mojang vanilla 26.2 sunucusu
`127.0.0.1:25568` üzerinde daha önce yapılan login, chunk delivery ve kontrollü
movement koşuları wire kabulünü doğrular; bunlar `server-conformance` olarak
ayrı kayıtlarda tutulur. Bu section lookup sonucu Java istemci oracle'ı ile
aynı sentetik payload üzerinde ölçüldüğü için bu kaydın kanıt sınıfı
`client-parity` olarak kalır.

## Açık kapsam

`ClientChunkCache.replaceWithPacketData`'nın gerçek Java client level/cache
nesnesiyle yürütülmesi, gerçek registry packet binding'i, block update,
section unload, ışık propagation ve block entity semantic state'i sonraki
P08 dilimlerindedir. Bu kayıt P08 bölümünü kapatmaz; canlı vanilla wire kabulü
istemci eşitliği yerine geçmez.
