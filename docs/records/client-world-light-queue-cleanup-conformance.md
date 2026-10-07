# P08.x — [server-conformance] Chunk unload light queue cleanup'i

Bu dosya, clientbound forget-chunk akışında chunk'a ait ışık state'inin ve
henüz uygulanmamış pending light update'ların temizlenmesini kaydeder. Canlı
kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2
dedicated server'ına bağlanır. Aynı girdinin Java istemcisiyle karşılaştırılması
yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientPacketListener.java:909-914`, forget-chunk paketinde chunk kaynağını
düşürür ve ışık temizliğini `ClientLevel` kuyruğuna ekler.
`ClientPacketListener.java:916-932`, ışık column'ını devre dışı bırakır, block ve
sky layer'larını `null` queued section verisiyle temizler ve dünya section'larını
empty durumuna alır. `ClientLevel.java:502-505` unload sırasında light engine'i
devre dışı bırakır. Bu cleanup, önceki light update runnable'larıyla aynı FIFO
işleme düzenini paylaşır.

## Sözleşme

- Chunk unload, ilgili chunk'ın sky ve block light section state'ini world
  kopyasından kaldırır.
- Aynı chunk için chunk henüz yüklenmeden bekleyen pending light update'lar
  unload sırasında silinir; daha sonra gelen chunk packet'i eski ışığı geri
  getirmez.
- Başka chunk'lara ait pending light update'lar korunur.
- Unload çağrısında chunk yoksa world removal sonucu `false` kalabilir; cleanup
  yine de pending ışık kayıtlarını temizler.
- Bu dilim light propagation, section dirty/rebuild ve render invalidation
  semantiğini uygulamaz.

## Rust doğrulaması

`ClientWorld::unload_chunk`, mevcut chunk column'ını kaldırmanın yanında aynı
koordinata ait `pending_light_updates` kayıtlarını filtreler. Böylece queue'da
bekleyen eski layer, gelecekteki `replace_chunk` çağrısına taşınmaz.

`unloading_a_chunk_discards_pending_light_updates_for_that_chunk` unit testi
yüklenmemiş chunk'a bir light update kuyruğa alır, unload ile kuyruğun
temizlendiğini ve aynı chunk daha sonra yüklendiğinde ışık state'i oluşmadığını
kontrol eder. Önceki unload testi mevcut chunk'ın section/query temizliğini
korur.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
`view-distance=5`, `simulation-distance=5`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Mevcut vanilla chunk unload probe'u 96 kontrollü hareket adımı ve her adımda
client-tick-end bildirimiyle çalıştırıldı:

```text
cargo run --locked --offline --example world_chunk_unload_probe -- 127.0.0.1:25568 P08xLive
```

Bu dilimin ilgili canlı smoke sonucu, vanilla forget-chunk paketlerinin Rust
world kopyasından chunk column'larını kaldırmaya devam ettiğini doğrular:

| Gözlem | Sonuç |
| --- | ---: |
| movement packets | `96` |
| clientbound unload packet ID | `37` |
| removed chunk columns | `75` |
| chunk packet / decoded / batch finished | `232 / 232 / 76` |
| unload sonrası world chunk / section | `157 / 3768` |
| ilk chunk query `(1,-2)` | `present_after_unload=false` |
| alınan paket sayısı | `6707` |

Bu canlı koşu cleanup değişikliğinin mevcut chunk unload akışını bozmadığını
gösterir. Yüklenmemiş chunk'a light update gelip hemen ardından forget-chunk
paketi gelmesini vanilla probe altyapısıyla üretmedim; pending cleanup dalı
Java kaynağına dayalı Rust unit testinde doğrulandı. Java client parity iddiası
yoktur.

## Açık kapsam

Light engine section status ve komşu propagation ayrıntıları, queued removal'ın
relight motoruyla birleşmesi, section dirty/rebuild, render/AO invalidation,
dynamic dimension temizliği ve `client-parity` karşılaştırması açık kalır.
