# P08.d — ClientChunkCache yaşam döngüsü — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `ClientChunkCache` görünüm aralığı, ring slot,
chunk replacement/drop ve boş section update-set davranışını Rust
`ClientWorld` cache'iyle karşılaştırır. Kanıt sınıfı `client-parity`'dir: aynı
özgün section payload'ları Java oracle'ına ve production Rust world yoluna
verilir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynak incelemesinde şu yollar esas alındı:

- `client/multiplayer/ClientChunkCache.java:38-42,59-77,99-125`, cache'in
  `max(2, serverChunkRadius) + 3` storage yarıçapını, in-range kapısını,
  ring slot lookup'ını, yeni/eski chunk ayrımını ve aynı chunk refresh yolunu
  tanımlar.
- `client/multiplayer/ClientChunkCache.java:131-159`, görünüm merkezinin
  doğrudan güncellenmesini ve radius değişince yeni storage oluşturup yeni
  aralıkta kalan chunk'ları kopyalama davranışını tanımlar.
- `client/multiplayer/ClientChunkCache.java:206-313`, floor-mod index'i,
  iki update buffer'ını, loaded/empty section ekleme-çıkarma setlerini ve
  `hasOnlyAir` geçişlerini tanımlar.
- `world/level/chunk/LevelChunkSection.java:85-88,163-186`, section header
  sayaçlarının ve `hasOnlyAir` kararının kaynağıdır.
- `core/SectionPos.java:219-224`, empty section setlerinde kullanılan packed
  section node değerini tanımlar.
- `client/multiplayer/ClientPacketListener.java:850-862,888-913,2421-2439`,
  chunk, forget, cache radius ve cache center packet'lerini bu cache'e bağlar.

## Davranış sözleşmesi

- Cache oluşturulduğunda storage yarıçapı `max(2, server_view_radius) + 3`,
  ring genişliği `2 * storage_radius + 1` olur. Başlangıç görünüm merkezi
  `(0, 0)`'dır.
- Chunk yalnızca merkezden iki eksende `storage_radius` uzaklık içindeyse
  kabul edilir. Kabul edilen koordinatın slotu
  `floorMod(z, view_range) * view_range + floorMod(x, view_range)` biçimindedir.
- Slot aynı chunk'a aitse section'lar yenilenir ve her section için
  `added_empty_sections` veya `removed_empty_sections` güncellenir. Slot başka
  bir chunk'a aitse eski chunk loaded/section setlerine çıkarma olayı olarak
  yazılır, sonra yeni chunk ekleme olayı yazılır.
- View center değişimi mevcut slotları kendiliğinden silmez. Radius'un storage
  genişliği değişirse Java yeni storage kurar, yalnız yeni aralıkta kalan
  chunk'ları taşır ve yeni update buffer'larını boş başlatır.
- `drop`, yalnız in-range ve slotta koordinatı doğrulanmış chunk'ı kaldırır.
  Chunk çıkarıldığında bütün section node'ları removed-empty setine girer;
  section emptiness callback'i ise yalnız in-range section'larda çalışır.
- Update setleri iki buffer'da tutulur. `flip` yeni buffer'a geçer ve yeni
  aktif buffer'ı temizler; önceki buffer'ın gözlemi korunur.

## Yeni dal ve varyant kararı

Bu, P08 section verisinin world lookup'a yazılmasından farklı olarak ilk kez
`ClientChunkCache`'in görünüm merkezi/radius ve slot yaşam döngüsünü
karşılaştıran daldır. Java source'ta ayrı `replace`, `drop`, radius rebuild ve
section emptiness dalları bulunduğu için tek parametrik senaryo bu dalları
aynı state trace'inde yürütür. Aynı kod yolunun yalnızca radius sabiti değişen
kopyaları yeni senaryo yapılmadı; `0→1` storage genişliği değişimi ve
`1→6` yeniden kurulum tek trace içinde gözlendi.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/chunk-cache-lifecycle` kaydı şunları birlikte ölçer:

- `server_view_radius=0` ile Java minimum storage radius ve negatif floor-mod
  index sınırları,
- air, non-empty ve fluid-count taşıyan section'larla ilk ekleme ve aynı
  chunk refresh'i,
- merkez değişiminden sonra `(0,0)` slotunun `(11,0)` ile değiştirilmesi,
- in-range/out-of-range `drop`,
- aynı storage genişliğinde radius güncellemesi ve genişlik değişince retained
  chunk kopyası,
- in-range ve out-of-range section emptiness callback'i ile buffer flip'i.

Java oracle, pinned JAR içindeki gerçek `LevelChunkSection.read` yolunu ve
`ClientChunkCache.Storage` sınıfının private `calculateStorageRange`,
`inRange` ve `getIndex` yollarını reflection üzerinden çalıştırır. Cache'in
private event setleri, kaynakta görülen `onChunkAdded`, `onChunkRemoved` ve
`refreshEmptySections` sırasına göre aynı Java state trace'inde tutulur.

Java referans kimliği:
`e6eb77d9eebfa749b3879e4297fe6742c0a2d3ec34046eb40e4c78d84fa0a215`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2965** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`src/world.rs` içindeki `ClientChunkCacheState`, Java storage radius/index,
slot map'i, double-buffered event setleri ve retained chunk rebuild'ini
uygular. `ClientWorld::replace_chunk` ve `unload_chunk` bu cache yapılandırılmış
olduğunda Java'nın in-range/slot doğrulamasını kullanır. Login'de server radius
ile cache kurulur; `set_chunk_cache_center`, `set_chunk_cache_radius` ve
forget-chunk packet'leri bağlantı akışında world state'ine uygulanır.

Debug adapter SHA-256 değeri
`b751833d5a08ed1bf8d95d4f9b71f7bef1c13feb45dca4eda4da4da599a1d78f` ile aynı
capture üzerinde **2965/2965** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`0085e04b8dd1972da039d1dff11a8a9d1946caf98ae27d6230778ad17aaba01d` ile aynı
capture üzerinde **2965/2965** eşleşti; fark `null` kaldı.

Rust doğrulama sonucu:

```text
cargo test --locked --offline --quiet
188 passed; 0 failed

python3 tools/behavior/test_runner.py
5 tests OK
```

## Vanilla server-conformance ayrımı

Orijinal Mojang vanilla 26.2 dedicated server, `127.0.0.1:25568` üzerinde
offline olarak kullanıldı. Cache packet bağından sonra canlı smoke sonucu:

```text
client_connection=ok
play_login=true
world_constructed=true
world_chunk_count=24
world_section_count=576
world_game_time=2
local_player_tick_count=2
```

Bu canlı koşu, Rust'ın vanilla login/chunk packet akışını kabul edip
`ClientWorld`'e yazdığını gösteren `server-conformance` kanıtıdır. Java
istemcisiyle aynı canlı state karşılaştırması olmadığı için bu kayıt içindeki
`client-parity` sonucunun yerine geçmez.

## Açık kapsam

Gerçek `ClientLevel` nesnesinin light engine invalidation ve tint cache yan
etkileri, block entity temizliğinin tüm semantiği, light propagation, biome
replace packet'i, entity section lifecycle'ı ve oynanabilir render/input
istemcisi sonraki P08 dilimlerindedir. Bu kayıt P08 bölümünü kapatmaz.
