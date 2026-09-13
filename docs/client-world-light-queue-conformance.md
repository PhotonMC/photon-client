# P08.w — [server-conformance] Unloaded chunk light update queue'su

Bu dosya, clientbound bağımsız light update chunk kopyası henüz yokken gelen
verinin kaybolmamasını ve ilgili chunk yüklendiğinde FIFO sırasıyla uygulanmasını
kaydeder. Canlı kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang
vanilla 26.2 dedicated server'ına bağlanır. Aynı girdinin Java istemcisiyle
karşılaştırılması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientPacketListener.java:850-862,2537-2558`, chunk ve bağımsız light update
paketlerinin doğrudan light engine'e değil `ClientLevel.queueLightUpdate` içine
FIFO runnable olarak bırakıldığını gösterir. `ClientLevel.java:167,274-288`
light update kuyruğunu tutar; `pollLightUpdates` her frame'de kuyruk boyutuna
göre en az 10 veya kalan işin onda biri kadar işi çalıştırır, 1000 ve üstünde
kalan kuyruğu tüketir.

`LayerLightSectionStorage.java:29,65-75,118-121,128-174,203-209`, section
verisinin `queuedSections` içinde bekleyebildiğini ve görünür storage hazır
olduğunda güncelleme sırasında storage'a taşındığını gösterir. Bu nedenle
bağımsız light update'ın chunk kopyasından önce ulaşması veri kaybı değildir.
`ClientChunkCache.java:99-124` chunk packet'i geldiğinde kopyayı oluşturur;
`ClientPacketListener.java:894-910` ise ışık section durumunu ayrıca etkinleştirir.

## Sözleşme

- Yüklü olmayan chunk için gelen geçerli bağımsız light update FIFO pending
  kuyruğuna kopyalanır ve packet işleme anında kaybolmaz.
- Aynı chunk için birden fazla pending update, yükleme sırasında giriş sırasıyla
  uygulanır; aynı layer ve section için son update görünür state'i belirler.
- Chunk kopyası oluşturulduğunda o chunk'a ait pending güncellemeler uygulanır;
  başka chunk'lara ait güncellemeler kuyrukta kalır.
- Geçersiz 2048 byte layer kuyruğa alınmaz ve codec uygulaması hata sonucu
  üretir.
- Dimension değişiminde eski world ile birlikte pending light kuyruğu da
  temizlenir.
- Bu dilim komşu block light propagation, relight priority queue, section
  dirty/rebuild işaretleri ve render/AO bağlantısını uygulamaz.

## Rust doğrulaması

`ClientWorld` pending light update'ları sahiplenilmiş mask, empty mask ve byte
layer'larıyla `VecDeque` içinde tutar. Yüklü chunk yoksa
`apply_light_updates` geçerli veriyi kuyruğa alır ve `queued` sonucu döndürür;
`replace_chunk` ilgili chunk'ı ekledikten sonra pending kayıtları FIFO olarak
`ClientLightSection` state'ine uygular. `pending_light_update_count` kuyruk
durumunu sorgulanabilir kılar. Dimension replacement eski pending kayıtları
temizler.

`light_updates_for_unloaded_chunks_wait_in_fifo_until_chunk_load` unit testi
aynı chunk'a iki farklı sky layer yollar, chunk yokken iki kaydın beklediğini
ve chunk yüklendiğinde ikinci layer'ın görünür olduğunu doğrular. Önceki light
state testleri de yüklü chunk, nibble sırası, empty layer ve unload davranışını
korur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
`view-distance=5`, `simulation-distance=5`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Kuyruk diliminin deterministik state koşusu Rust unit testidir. Vanilla
sunucuda mevcut yüklü chunk light update yolu için bağlantı açıkken hedef blok
iki ayrı server tick'inde önce air, sonra glowstone yapıldı; iki komut aynı
tick'e birleştirilmedi:

```text
cargo run --locked --offline --example world_light_update_probe -- 127.0.0.1:25568 P08wLive3
setblock 25 80 -36 minecraft:air
setblock 25 80 -36 minecraft:glowstone
```

Geçerli koşunun sonucu:

| Gözlem | Sonuç |
| --- | ---: |
| light update packet count | `1` |
| applied light sections | `1` |
| target block light | `Some(15)` |
| first update | `chunk=(-3,-8), sky=0, block=1, block layer=2048` |
| loaded light sections | `1855` |
| world chunk / section | `157 / 3768` |
| received packet count | `10400` |

Bu canlı koşu, queue değişikliğinin mevcut yüklü chunk update yolunu bozmadığını
ve vanilla `LIGHT_UPDATE` layer'ının world state'ine uygulanabildiğini gösterir.
Yüklenmemiş chunk'a gelen bağımsız packet sırasını bu probe altyapısıyla canlı
olarak oluşturmadım; pending/FIFO davranışı Java kaynak dalına göre Rust unit
testinde doğrulandı. Bu kayıt Java client parity iddiası değildir.

## Açık kapsam

Tam light propagation/relight engine'i, queued section'ın komşu propagation'ı,
light removal sırasında section temizliği, section dirty/rebuild işaretleri,
render/AO bağlantısı, heightmap semantiği ve `client-parity` karşılaştırması
açık kalır.
