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

## Sapma nerede: dolaylı aydınlanan yüzeylerde

Farkın **nerede** olduğu sebebe işaret ediyor. Kareyi 6x4 bölgeye bölüp
ortalama mutlak farka bakınca alt sıra — glowstone zemin — her ölçümde tam
`0.00` çıkıyor; fark yalnızca duvarlarda:

```
 0.96  1.48  1.57  1.57  1.48  0.97
 2.53  2.57  2.58  2.57  2.36  1.52
 2.62  1.67  1.56  1.56  1.40  1.18
 0.00  0.00  0.00  0.00  0.00  0.00   <- glowstone zemin
```

Işık **kaynağı** birebir aynı, **yayılan** ışık değişiyor. İstemci ışık
değerlerini chunk mesh'ine pişirdiği için, mesh ışık yayılımı tamamlanmadan
derlendiğinde sonuç kalıcı olarak farklı kalır ve hiçbir şey yeniden derlemeyi
tetiklemediği için koşu boyunca öyle durur. Bu, koşu içi kararlılık ile
koşular arası sapmanın neden bir arada olduğunu açıklayan tutarlı bir
hipotezdir. Bu ölçüm tuş enjeksiyonu içermez; mekânsal kanıt sağlamdır.

### Hipotez sınandı ve doğrulanamadı

Yakalamadan önce `F3+A` ile bütün chunk'ları yeniden derletmek hipotezin
doğrudan testidir. **Test sonuç vermedi**, çünkü testin kendisi ölçümü bozuyor.

Tuşu iletmek için pencerenin öne alınması gerekiyor ve bu, görüşü bir derecenin
altında döndürebiliyor. Bölge ortalamaları ve ufuk satırı bu dönüşü
**yakalamıyor**: dört koşuda zemin sınırı satırı `474`, duvar ve zemin bölge
ortalamaları birbirine yakın kaldı. Yakalayan şey zemin dokusunun yatay fazı:

| Koşu | `y=400` parlak tepe x'leri |
| --- | --- |
| GL, yeniden derleme yok | `41, 66, 140, 165, 239, 264` |
| GL, yeniden derlemeli | `41, 66, 140, 165, 239, 264` |
| VK, yeniden derleme yok | `41, 66, 140, 165, 239, 264` |
| VK, yeniden derlemeli | `43, 75, 174, 273, 362, 365` |

Son satırdaki koşuda kamera oynamış; o kare başka hiçbir kareyle
karşılaştırılamaz. Çalıştırıcı artık bu parmak izini yeniden derlemenin iki
yanında alıyor ve değişmişse koşuyu reddediyor. Dört Vulkan denemesinden biri
bu şekilde reddedildi.

Kamera koruması açıkken üç Vulkan koşusu:

| Karşılaştırma | Max kanal | Ortalama kanal |
| --- | ---: | ---: |
| A ↔ B | `4` | `0.64` |
| A ↔ C | `8` | `1.63` |
| B ↔ C | `5` | `0.99` |

Yeniden derleme olmayan Vulkan koşularındaki `6` / `1.29` ile aynı
mertebede, bir çiftte daha kötü. **Zorunlu yeniden derleme koşular arası
sapmayı güvenilir biçimde azaltmıyor.**

Bu kaydın önceki sürümü, yeniden derlemenin tabanı `7`'den `2`'ye indirdiğini
bildiriyordu. O ölçüm kamera korumasından önce alınmıştı ve üç OpenGL
koşusunun benzer biçimde etkilenmiş olmasına dayanıyordu; **geri alınmıştır**.
Sebep hipotezi mekânsal kanıtla ayakta duruyor, ama giderilebilirliği
gösterilmedi.

### Hipotez ikinci yolla doğrulandı: varyans yayılım miktarıyla ölçekleniyor

Tuş enjeksiyonu güvenilmez olduğu için hipotez sahneyi değiştirerek sınandı.
`emissive` sahnesi odanın kabuğunu da `glowstone` yapar: her görünür yüzey ışık
kaynağıdır, yayılacak gradyan neredeyse kalmaz. Aynı çalıştırıcı, aynı sabit
kamera, tuş gönderilmeden üç koşu:

| Sahne | Farklı piksel | Max kanal | Ortalama kanal |
| --- | ---: | ---: | ---: |
| `room` (tek ışık kaynağı, yayılımla aydınlanan duvarlar) | %71 | `7` | `1.29`–`1.37` |
| `emissive` (her yüzey ışık kaynağı) | %10–11 | `3`–`7` | `0.11`–`0.31` |
| `emissive`, ambient occlusion kapalı | %3–11 | `1`–`6` | `0.01`–`0.26` |

Yayılım azaldıkça varyans azalıyor: farklı piksel oranı yedi kat, ortalama
kanal farkı beş kat düştü. Ambient occlusion'ı kapatmak ek kazanç vermedi, yani
kalan bileşen AO değil.

Kalan farkın yeri de aynı yöne işaret ediyor. `emissive` + AO kapalı durumda
bölgesel harita merkezde ve üst/alt şeritlerde tam `0.00`, fark yalnızca sol ve
sağ kenarlarda:

```
 0.00  0.00  0.00  0.00  0.00  0.00
 1.44  0.30  0.00  0.00  0.10  0.54
 2.16  0.44  0.00  0.00  0.18  0.90
 0.00  0.00  0.00  0.00  0.00  0.00
```

Bunlar odanın köşeleri: duvarların hepsi ışık kaynağı olsa da köşedeki hava
bloklarının ışık seviyesi hâlâ yayılımla belirleniyor. Ayrışan pikseller
eşleşenlerden daha karanlık (ortalama `97.5` karşı `130.9`), yani gradyanın
olduğu yerde.

Böylece sebep iki bağımsız yolla doğrulanmış oldu: farkın **yeri** (ışık kaynağı
bit-bit aynı, yayılımla aydınlanan yüzeyler değil) ve farkın **büyüklüğünün
sahnedeki yayılım miktarıyla ölçeklenmesi**. Hiçbiri tuş enjeksiyonuna
dayanmıyor.

**Giderilmedi.** Kamerayı bozmadan chunk yeniden derlemesini tetikleyen bir yol
bulunmadıkça, ya yayılımın tamamlandığını gösteren bir bekleme koşulu bulunmalı
ya da tolerans sahnenin aydınlatma karmaşıklığına bağlı olarak tanımlanmalı.
İkincisi [davranış testi sözleşmesinin](behavior-testing.md) "evrensel epsilon
varsayma" kuralıyla ancak gerekçesi ve sınırı yazılırsa bağdaşır.

## OpenGL ↔ Vulkan farkı: 22 piksel, deterministik

`delta > 8` olan pikseller sayıldığında tablo nettir:

| Karşılaştırma | `delta > 8` piksel |
| --- | ---: |
| GL R1 ↔ GL R2 | `0` |
| GL R1 ↔ GL R3 | `0` |
| VK V1 ↔ VK V2 | `0` |
| GL R1 ↔ VK V1 | `22` |

Her backend bu piksellerde kendisiyle tutarlı; ayrışma yalnızca backend'ler
arasında. Yani gürültü değil, tekrarlanabilir bir fark. Yirmi iki pikselin on
ikisi tek satırda (`y=361`) ve `75` piksel düzenli aralıkta duruyor — zeminin
neredeyse kenarından görüldüğü satır. Değerler aynı dokunun komşu iki texel'i:

```
(389,361)  gl=[117, 78, 41]   vk=[248,228,184]
(464,361)  gl=[248,228,184]   vk=[117, 78, 41]
```

Yön dönüşümlü. Bu, farklı bir aydınlatma veya geometri modeli değil, sıyırma
açısında texel sınırına denk gelen örneklemenin iki backend'de farklı
yuvarlanmasıdır.

**Karar için anlamı:** Vulkan hedeflemek, bu sahnede Java'nın varsayılan
OpenGL yapılandırmasından `22` pikselde kalıcı olarak ayrılmak demektir.
Ortalamada bedeli yok, ama sıfır da değil. [Backend kararında](render-backend-decision.md)
yazılan "pinned referans yapılandırması `preferredGraphicsBackend=vulkan`
olmalıdır" şartı artık varsayım değil, ölçülmüş bir gerekliliktir.

## Bundan çıkan kural

Tek bir Java koşusuna karşı **bit-bit piksel eşitliği hedeflenemez**, çünkü
referans kendisiyle bile bit-bit örtüşmüyor. Render dilimleri için sıra şudur:

1. Sapmanın yeri kesin (dolaylı aydınlanan yüzeyler) ve hipotez tutarlı, ama
   **giderilmedi**. Zorunlu yeniden derleme işe yaramıyor ve tuş enjeksiyonu
   kamerayı bozabildiği için bu yolla sınanamıyor. Kamerayı hiç kıpırdatmayan
   bir yeniden derleme tetiği ya da ışık yayılımının bittiğini logdan okuyacak
   bir bekleme koşulu gerekiyor.
2. OpenGL↔Vulkan izole farkların kaynağı açıklandı: sıyırma açısında texel
   sınırı yuvarlaması, `22` piksel, backend başına deterministik. Kapsam
   genişletilmeli — başka sahne, çözünürlük ve mipmap ayarında kaç piksel
   olduğu ölçülmedi.
3. Ancak bundan sonra bir render karşılaştırma ölçütü yazılabilir.

Bu ölçüm üç OpenGL ve iki Vulkan koşusundan gelir; tek makine, tek sürücü, tek
çözünürlük. Başka ortamlarda tekrarlanmadan genel bir tolerans olarak
kullanılamaz. [Davranış testi sözleşmesindeki](behavior-testing.md) "evrensel
epsilon varsayma" kuralı burada da geçerlidir: bu sayılar bir tolerans değil,
bir engelin ölçüsüdür.
