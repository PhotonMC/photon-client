# P08.e — [server-conformance] ClientWorld entity ticking eligibility

Bu kayıt, client world kopyasındaki entity'nin bulunduğu chunk'ın yüklenme
durumuna göre ticking uygunluğunun güncellenmesini kapsar. Canlı kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'ına bağlanır ve chunk yaşam döngüsünü tüketir. Rust unit testleri Java
kaynağından çıkarılan state sözleşmesini korur; Java istemcisiyle aynı girdide
karşılaştırma yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientLevel.java:146-147` ticking entity listesini ve geçici entity section
deposunu ayrı tutar. `ClientLevel.java:450-459` yalnızca bu ticking listesindeki
entity'leri tick eder ve listenin sorgulanabilir olduğunu gösterir.

`ClientLevel.java:502-511`, bir chunk unload edildiğinde block entity'leri
temizler, ışığı devre dışı bırakır ve ilgili entity section'ını
`stopTicking` ile tracked durumuna alır. Chunk tekrar yüklendiğinde
`onChunkLoaded` aynı section için `startTicking` çağırır.

`TransientEntitySectionManager.java:13-65`, chunk başına ticking işaretini
tutar. `startTicking` ve `stopTicking`, section görünürlüğünü değiştirir ve
`isAlwaysTicking` olmayan entity'ler için `onTickingStart`/`onTickingEnd`
callback'lerini çağırır. Yeni entity, section zaten ticking ise listeye alınır;
always-ticking sınıflandırması ayrıca uygulanır.

`TransientEntitySectionManager.java:83-121`, entity section değiştirince eski
ve yeni section durumlarını karşılaştırır. Entity, ticking section'dan tracked
section'a geçtiğinde listeden çıkar; ters yönde tekrar eklenir.

`ClientLevel.java:1295-1301` callback'lerin ticking entity listesine ekleme ve
çıkarma işini yaptığını doğrular. Bu dilim bu listenin üyeliğini saklar;
`ClientLevel.tickEntities` içindeki gerçek entity tick gövdesini henüz
çalıştırmaz.

Vanilla server tarafındaki `ChunkMap.java:1326-1426`, entity tracker görünürlük
mesafesini oyuncunun view distance'ı ile sınırlar. Bu yüzden canlı server
akışında bir entity çoğunlukla `ForgetLevelChunk` paketinden önce
`RemoveEntities` ile düşer; bu sıra, client world unload callback'inin
`stopped_entity_count` değerini canlı olarak görmesini garanti etmez.

## Sözleşme

- `ClientWorld` entity map'i chunk unload edildiğinde entity'yi otomatik olarak
  silmez. Entity'nin bulunduğu chunk kaldırılırsa entity `is_ticking=false`
  olur.
- AddEntity mevcut yüklü chunk'a düşüyorsa entity ticking kabul edilir; chunk
  henüz yoksa tracked olarak tutulur. AddEntity packet'i on-ground alanı
  taşımadığı için bu state değişmez.
- PositionSync, Teleport ve konum taşıyan MoveEntity packet'leri son konuma
  göre entity'nin chunk anahtarını yeniden hesaplar. Hedef chunk yüklüyse
  ticking true, değilse false olur; yalnız rotation taşıyan move dalı ticking
  state'ini değiştirmez.
- Bir chunk tekrar replace edildiğinde o chunk'ta bulunan entity'ler tekrar
  ticking olarak işaretlenir.
- RemoveEntities entity'yi global map'ten kaldırır; ticking state'inin
  güncellenmesi için ayrıca chunk unload gerekmez.
- Bu dilim gerçek entity tick/fizik entegrasyonunu, always-ticking entity
  türlerini, section içi görünürlük ayrımını, interpolation'ı, passenger
  ilişkisini ve dimension/respawn geçişini kapsamaz.

## Rust doğrulaması

`ClientEntity` artık `is_ticking()` ile sorgulanabilir. `ClientWorld` entity
eklerken, chunk değiştirirken, chunk unload/reload ederken ve entity konumu
değişirken bu state'i günceller. `ChunkUnloadObservation`, canlı packet
akışındaki tracked/ticking/stopped sayımlarını kaydeder.

`entities_are_queryable_and_replaced_by_entity_id` unit testi şu geçişleri
kontrol eder: yüklü chunk'a ekleme, unload sonrası entity'nin map'te kalıp
ticking'den çıkması, reload sonrası tekrar ticking olması, hedef chunk'a göre
PositionSync durumu ve MoveEntity ile yüklü hedef chunk'a geçiş.

Canlı example yalnızca en az bir chunk unload'unun `ClientWorld`'den
kaldırıldığını başarı sayar. `stopped_entity_count=0` sonucu başarısızlık
sayılmaz; vanilla tracker'ın `RemoveEntities` paketini önce gönderebildiği bu
durumda live stop callback'i için gözlem yoktur ve sonuç açıkça raporlanır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

```text
cargo run --locked --offline --example world_entity_ticking_probe -- 127.0.0.1:25568 TickFinal
```

Probe 96 küçük konum adımı gönderir; her adım client tick-end paketiyle
tamamlanır. Ana vanilla koşusunun çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `chunk_unload_packet_count` | `24` |
| `chunk_unload_applied_count` | `24` |
| `world_entity_count` | `56` |
| `ticking_entity_count` | `56` |
| `tracked_stopped_entity_count` | `0` |
| `unloads_with_stopped_entities` | `0` |
| `world_chunk_count` | `157` |
| `world_section_count` | `3768` |
| `received_packet_count` | `9784` |

Bu koşu, vanilla'nın gönderdiği chunk unload akışının Rust world kopyasında
uygulandığını ve probe'un movement/chunk batch sınırlarını tamamladığını
gösterir. `stopped_entity_count=0`, canlı akışta entity'nin unload anından
önce tracker tarafından kaldırıldığını gösteren kapsam sınırlamasıdır; Java
client state eşitliği iddiası değildir.

## Açık kapsam

Gerçek `ClientLevel.tickEntities` yürütmesi, always-ticking entity türleri,
section tabanlı entity getter'ları, entity fizik/motion entegrasyonu,
interpolation, passenger ilişkileri, dimension/respawn, chunk unload sırasında
Java block entity/light yan etkilerinin birlikte yürütülmesi, render state'i ve
`client-parity` karşılaştırması açık kalır.
