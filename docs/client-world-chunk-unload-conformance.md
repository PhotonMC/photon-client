# P08.d — [server-conformance] ClientWorld chunk unload

Bu kayıt, vanilla sunucusunun oyuncu görüş alanı değiştiğinde gönderdiği
chunk-unload paketinin Rust istemci dünya kopyasındaki chunk column'ı ve
section cache'ini kaldırmasını kapsar. Kanıt sınıfı `server-conformance`'dır:
Rust paket akışını Mojang vanilla 26.2 sunucusuna bağlayıp tüketir. Java
istemcisiyle aynı girdide state karşılaştırılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`ClientboundForgetLevelChunkPacket.java:9-17`, `ChunkPos.pack` değerini tek bir
64 bit alan olarak taşır; alt 32 bit X, üst 32 bit Z koordinatıdır.
`ClientPacketListener.java:909-918` bu pakette chunk kaynağını düşürür ve
ışık verisinin kaldırılmasını kuyruğa alır. Sunucu tarafında
`ChunkMap.java:1074-1122` eski ve yeni izleme görünümünün farkında çıkan
column'ları `PlayerChunkSender.dropChunk` yoluna verir;
`PlayerChunkSender.java:43-47` pending olmayan canlı bir chunk için unload
paketini gönderir.

Vanilla client her oyun tick'inin sonunda
`Minecraft.java:1844` üzerinden boş `ServerboundClientTickEndPacket` yollar.
Sunucu `ServerGamePacketListenerImpl.java:2201-2207` bu sınırı işler ve
`MinecraftServer.java:1148` sonrasında bekleyen chunk batch'lerini ilerletir.
Bu nedenle probe, hareket adımları arasında client-tick-end paketini de
gönderir.

## Sözleşme

- Başlangıç chunk teslimi `ClientWorld` içine yazılır.
- Oyuncu yeni bir chunk görünüm merkezine geçtiğinde gelen clientbound packet
  ID `37`, 8 baytlık packed `ChunkPos` olarak çözülür.
- Unload edilen `(chunk_x, chunk_z)` column'ı `ClientWorld` deposundan ve
  section cache'inden tamamen çıkarılır; aynı koordinat için block-state query
  sonucu `None` olur.
- Bu dar dilim yalnız chunk yaşam döngüsünü kapsar. Işık nibble verisinin
  kaldırılması, block entity temizliği, entity yaşam döngüsü ve render
  invalidation ayrı işlerdir.

## Rust doğrulaması

`parse_forget_level_chunk_packet` Java packed chunk koordinatını ve trailing
payload hatasını birim testinde kontrol eder. `ClientWorld` birim testi de
column replacement ve unload sonrası section/query durumunu kontrol eder.

`world_chunk_unload_probe` başlangıç world state'ini aldıktan sonra 96 küçük
hareket adımı gönderir. Her adım client tick-end ile tamamlanır. Probe, hiç
unload gelmezse veya gelen unload'ların hiçbiri world kopyasından silinmezse
başarısız çıkar; böylece eksik canlı kanıt başarı sayılmaz.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed
`776-unload`, creative test modu, `view-distance=2`,
`simulation-distance=2`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

```text
cargo run --locked --offline --release --example world_chunk_unload_probe -- 127.0.0.1:25570 UnloadR2
```

İlgili sonuç:

| Gözlem | Sonuç |
| --- | --- |
| movement paketleri | `96` |
| clientbound unload packet ID | `37` |
| `ClientWorld`'den kaldırılan chunk column | `54` |
| chunk packet / decoded / batch finished | `103 / 103 / 38` |
| unload sonrası world chunk / section sayısı | `49 / 1176` |
| ilk chunk `(0,-3)` sorgusu | `present_after_unload=false` |

Server logunda protocol error, movement warning veya timeout görülmedi.
Sonuç, vanilla'nın gönderdiği chunk yaşam döngüsü paketinin Rust world
kopyasına uygulandığını gösterir; Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Dinamik dimension/respawn, ışık nibble state'i ve unload sırasında ışık
temizliği, block entity kopyası, entity deposu/yaşam döngüsü, prediction ile
server-verified state çakışması ve render invalidation açık kalır.
