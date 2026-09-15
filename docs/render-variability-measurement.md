# P15.a.2 — Java referansının kendi değişkenliği — [ölçüm, parity iddiası yok]

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Ortam:** AMD Radeon RX 9060 XT, Mesa 26.2.2-arch3.2, XWayland, 854x480

Bu kayıt eşitlik kanıtı değildir. Render bölümünün tolerans tabanını ölçer:
**aynı sabit sahnede Java'nın kendisi kendisiyle ne kadar örtüşüyor?** Bu sayı
bilinmeden hiçbir render karşılaştırması yorumlanamaz.

## Sabit sahne

`tools/render/scene.py build` pinned dedicated server ile seed `776` üzerinde
kapalı bir oda üretir: `stone` kabuk, `glowstone` zemin, dört köşede farklı
malzeme (meşe tahta, demir blok, kitaplık, kırmızı yün). Gökyüzü, bulut, su,
hava, gece/gündüz döngüsü, entity ve rastgele tick yok. `place` ise oyuncunun
konumunu, bakışını, hareketini ve oyun modunu kaydedilmiş oyuncu dosyasına
yazar.

`tools/render/reference.py launch --scene parity-room` sahneyi her koşuda
temiz kopyalar ve `--quickPlaySingleplayer` ile menüsüz açar.

## Kameranın sabitlenmesi

İlk denemede oyuncu odanın **çatısında** başladı: istemcinin güvenli-spawn
araması en yüksek katı bloğa çıkıyor ve mühürlü kabuk bunu engellemiyor.
Kaydedilmiş oyuncu verisi bu aramayı geçersiz kıldığı için konum doğrudan o
dosyaya yazıldı.

İkinci denemede kare koşu içinde sabitti ama koşular arasında parlaklık
monoton artıyordu (56.0 → 58.8 → 62.5 → 67.2). Sebep: oyuncu zeminin bir blok
üstüne konmuştu ve yavaşça alçalıyordu; sahne değil **kamera** kayıyordu.
Ayaklar zemine oturtulup `Motion` sıfırlanınca koşu içi kareler bit-bit aynı
oldu.

Bu iki hata ölçümün kendisinden daha öğreticidir: sabit görünen bir sahne, sabit
olmayan bir kamerayla sessizce değişken hale geliyor.

## Ölçüm

Üç OpenGL, iki Vulkan koşusu; her koşuda 20 saniye yerleşme sonrası yakalama.

| Karşılaştırma | Farklı piksel | En büyük kanal farkı | Ortalama kanal farkı |
| --- | ---: | ---: | ---: |
| Koşu içi (20s vs 28s) | `0` | `0` | `0` |
| OpenGL R1 ↔ R3 | %10.13 | `2` | `0.034` |
| OpenGL R1 ↔ R2 | %71.22 | `7` | `1.332` |
| OpenGL R2 ↔ R3 | %71.23 | `7` | `1.366` |
| Vulkan V1 ↔ V2 | %71.14 | `6` | `1.288` |
| OpenGL ↔ Vulkan | %64.9–70.5 | `169` | `0.47–0.83` |

## Okunuşu

**Koşu içi kararlılık tamdır.** Kamera sabitlendikten sonra aynı oturumda
saniyeler arayla alınan kareler bit-bit aynıdır. Yakalama yolu ve sahne
kendi içinde deterministiktir.

**Koşular arası kararlılık yoktur ve gürültü değildir.** R1 ile R3 neredeyse
örtüşürken R2 ikisinden de yaklaşık `1.35` sapıyor. Dağılım tek yönlü ve iki
durumlu: rastgele GPU gürültüsü değil, koşu başına farklı oturan sistematik bir
durum. Aynı desen Vulkan'da da görülüyor, yani backend'e özgü değil. Aday
sebepler — henüz hiçbiri doğrulanmadı — ışık yayılımının yakalama anında farklı
noktada tamamlanmış olması, atlas/mipmap üretim sırası ve dünyaya girişteki
solma evresi.

**Backend farkı ortalamada koşular arası farktan küçüktür.** OpenGL ile Vulkan
arasındaki ortalama kanal farkı `0.47–0.83`, aynı backend'in iki koşusu
arasındaki `1.29–1.37`'den düşük. Erken bir dağılım ölçümünde piksellerin
ezici çoğunluğu tam `0`, kalanı `±1` çıkmıştı. Bu haliyle
[backend seçimi](render-backend-decision.md) ortalamada bedelsiz görünüyor.

**Ama izole pikseller çok ayrışıyor.** OpenGL↔Vulkan en büyük kanal farkı
`169`. Sayıca çok az piksel, fakat büyüklüğü rastgele yuvarlama ile
açıklanamaz. Bu piksellerin nerede olduğu ve neden ayrıştığı açıklanmadan
"iki backend eşdeğerdir" denemez.

## Sapmanın sebebi: ışık oturmadan derlenen chunk mesh'leri

Farkın **nerede** olduğu sebebi verdi. Kareyi 6x4 bölgeye bölüp ortalama mutlak
farka bakınca alt sıra — glowstone zemin — her ölçümde tam `0.00` çıkıyor;
fark yalnızca duvarlarda. Yani ışık **kaynağı** birebir aynı, **yayılan** ışık
değişiyor. İstemci chunk mesh'ine ışık değerlerini pişirdiği için, mesh ışık
yayılımı tamamlanmadan derlendiğinde sonuç kalıcı olarak farklı kalıyor ve
hiçbir şey yeniden derlemeyi tetiklemediği için koşu boyunca öyle duruyor. Bu,
koşu içi kararlılık ile koşular arası sapmanın neden bir arada olduğunu
açıklıyor.

Test: yakalamadan önce `F3+A` ile bütün chunk'ları yeniden derletmek.

| Koşul | Koşular arası max kanal | Ortalama kanal |
| --- | ---: | ---: |
| Yeniden derleme yok (3 koşu) | `7` | `1.29`–`1.37` |
| Yeniden derleme, 6 s bekleme (3 koşu) | `2` | `0.13`–`0.32` |
| Yeniden derleme, 20 s bekleme (2 koşu) | `4` | `0.82` |

İki durumlu `~1.35` sapma kayboluyor. Zemin bölgeleri yine tam `0.00` kalıyor,
yani kamera kaymadı.

Bekleme süresini uzatmak **iyileştirmiyor**; 20 saniyede sonuç 6 saniyedekinden
kötü çıktı. Buradan iki şey çıkar: stale baked lighting farkın büyük bileşeni
ama tamamı değil, ve kalan bileşen zamanla yakınsamıyor. Örneklem küçüktür
(koşul başına iki-üç koşu), bu yüzden `2` ile `4` arasındaki fark tek başına
anlamlı sayılmamalıdır.

### Tuş enjeksiyonunda yakalanan tuzak

`F3+A` ilk denemede iki ayrı tuş vuruşu olarak gönderildi. Oyun `A`'yı spectator
sola kaydırma olarak okudu; kamera yer değiştirdi ve karşılaştırma ortalaması
`24.9`'a fırladı. Akor, `F3` basılı tutulup `A`'ya basılarak gönderilince sorun
kalktı. Sabit kameraya dayanan bir ölçümde tuş göndermek, tuşun oyun içi
anlamını hesaba katmadan yapılamaz.

## Bundan çıkan kural

Tek bir Java koşusuna karşı **bit-bit piksel eşitliği hedeflenemez**, çünkü
referans kendisiyle bile bit-bit örtüşmüyor. Render dilimleri için sıra şudur:

1. İki durumlu sapmanın sebebi bulundu ve büyük ölçüde giderildi: chunk
   mesh'leri ışık yayılımı tamamlanmadan derleniyor. Yakalamadan önce zorunlu
   yeniden derleme tabanı `7`'den `2`–`4`'e indiriyor. Kalan bileşen hâlâ
   açıklanmadı ve beklemekle kapanmıyor.
2. OpenGL↔Vulkan izole büyük farkların kaynağı açıklanmalı.
3. Ancak bundan sonra bir render karşılaştırma ölçütü yazılabilir.

Bu ölçüm üç OpenGL ve iki Vulkan koşusundan gelir; tek makine, tek sürücü, tek
çözünürlük. Başka ortamlarda tekrarlanmadan genel bir tolerans olarak
kullanılamaz. [Davranış testi sözleşmesindeki](behavior-testing.md) "evrensel
epsilon varsayma" kuralı burada da geçerlidir: bu sayılar bir tolerans değil,
bir engelin ölçüsüdür.
