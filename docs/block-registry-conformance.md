# P16.s — Blok kayıt defteri — [client-parity] kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir. 1196 blok, 121 özellik, 32366 blok durumu ve
28 tonlama girdisinin tamamı her parity koşusunda referansın kendi kayıt
defteriyle karşılaştırılıyor.

## Engel neydi

Kayıt defteri birçok dilimi aynı anda kilitliyordu: varyant seçiminin
doğrulanması, tonlama tablosu, yüz örtülmesi ve sunucudan gelen blok durumu
numaralarının çözülmesi. Daha önce denendiğinde oracle süreci düşmüştü ve
"registry bootstrap oracle'da çalışmıyor" diye kaydedilmişti.

**O teşhis yanlıştı.** Bootstrap çalışıyordu; sorun `Bootstrap.bootStrap()`
sonundaki `wrapStreams()` çağrısıydı. O çağrı `System.out`'u oyunun
günlükçüsüne besleyen bir akışla değiştiriyor ve sağlayıcı yokken çıktıyı
yutuyor. Oracle JSON'unu oradan yazdığı için bootstrap'tan sonraki bütün
kayıtlar kayboluyordu — 9 kaydın 7'sinin gelmesinin sebebi buydu.

Çözüm iki satır: orijinal akışı bootstrap'tan önce saklayıp hemen sonra geri
koymak, ve `Oracle`'ın kayıtları en baştaki akışa yazması.

İkinci çarpışma: `SharedConstants.setVersion` ikinci bir **farklı** sürüm
nesnesini reddediyor. `math` ailesi zaten kendi nesnesini kuruyordu, yeni
`block` ailesi `tryDetectVersion()` çağırıyordu ve ikisi birlikte koşunca
ikincisi patlıyordu. Sürüm nesnesi `Oracle`'a taşındı, ikisi de onu kullanıyor.

## Ölçülen iki olgu

Tabloyu küçük tutan şey bu ikisi ve ikisi de 1196 bloğun ve 32366 durumun
hepsinde doğrulandı:

1. Blok durumu numaraları **kayıt defteri sırasında kesintisiz** ilerliyor.
   Yani her blok için yalnızca başladığı numara saklanıyor; boşluk yok.
2. Bir bloğun durumları özelliklerinin **kartezyen çarpımı**, sıralama
   özellik sırasında ve **son özellik en hızlı** değişiyor. Yani bir numara
   tabloya bakmadan özelliklere geri çözülebiliyor.

İkisi de `src/world/block_registry.rs` içindeki testlerle korunuyor; ikinci
test 32366 durumun her birini ileri ve geri çeviriyor.

## Renk haritası tuzağı

İlk yakalamada `grass_block` tonlaması `00000000` çıktı. Sebep referansın
davranışı değil, headless ortam: `GrassColor` renk haritasını oyunun
varlıklarından yüklüyor ve yüklenmemişken dizi sıfır dolu. Haritalar pinned
JAR'ın içinde ve JAR zaten sınıf yolunda, bu yüzden oracle üçünü de oradan
okuyup kuruyor. Sonra `grass_block` `ff7cbd6b` veriyor.

Bu, "referanstan okudum" ile "referansın gerçekten yaptığı" arasındaki farkın
somut bir örneği; kaydedilmesinin sebebi de bu.

## Girdiler

`block/state-definitions` ve `block/tints`, her biri tek satır. Girdi küçük
çünkü oracle kayıt defterini kendisi geziyor; **beklenen çıktı** bütün tablo.

## Sonuçlar

Java ve Rust her bloğun adını, kayıt numarasını, özelliklerini (ad, tür,
değerler ve sıraları), varsayılan durumunu, durum sayısını ve başlangıç
numarasını aynı verdi. Tonlama tablosunun 28 girdisi ve iki katmanlı iki
bloğu (`pink_petals`, `wildflowers`) da eşleşti.

Koşu: `python3 tools/behavior/run.py capture` ardından
`compare --adapter target/debug/examples/behavior_adapter` → `parity_passed`,
3904 senaryo / 13 aile.

## Neyi açtı

- Tonlama tablosu artık tahmin değil: [P16.o](block-tint-conformance.md)
  "pinlenmemiş" diye işaretlenmişti, o boşluk kapandı. Daha önce dışarıda
  bıraktığım yapışık sap blokları da tabloda (`ffe0c71c`).
- Sonda blok özelliklerini kayıt defterine karşı doğruluyor: `facing=nowhere`
  artık sessizce yanlış varyant seçmek yerine hata veriyor. Verilmeyen bir
  özellik varsayılan durumun değerini alıyor.
- Blok durumu numarası hesaplanabiliyor (`block_state_id`), yani sunucudan
  gelen bir yığın bölümünü çözmenin önü açık.

## Açık kapsam

`VariantSelector.predicate` ve `Block.shouldRenderFace` hâlâ pinlenmedi.
İkisi de artık **mümkün** — kayıt defteri elde — ama ayrı dilimler.

Tonlamanın biyom ortalamalı yüzü (`colorInWorld`) hâlâ bir dünya istiyor.
