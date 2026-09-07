# İlk davranış testleri: implementasyondan önceki sözleşmeler

Bu belge P00, P01.a, P02.a ve P03.a için çalıştırılabilir testlere dönüştürülecek ilk sözleşmeleri tanımlar. [Kalıcı çalıştırıcı](behavior-runner.md) ile 180 sayısal/vektör, 22 zamanlayıcı ve 144 wire senaryosu hazır. Bu ilk corpus tam P01/P02/P03 kapsamı değildir; Rust implementasyonu yok. Referans kimliği [envanterde](reference-baseline.md) sabitlenmiştir.

## P00 — Java referansı ve karşılaştırıcı

Java tarafında yeni oyun algoritmaları yazmak yerine `.data/minecraft/runtime/client-26.2.jar` içindeki sınıflar çağrılacak. Küçük, özgün Java adaptörü yalnızca girdi kurma, çağrı ve gözlem yapacak. Decompile edilmiş algoritmayı test koduna kopyalamak bağımsız referans oluşturmaz.

Yerel yoklama, `DeltaTracker.Timer`, `Vec2` ve `VarInt` çağrılarının mevcut Java 25 ve `runtime/libs/*` ile grafik arayüzsüz çalışabildiğini gösterdi. İlk kalıcı çalıştırıcı bu yolu kullanabilir. Java wrapper'ı depoda tutulabilir; orijinal JAR, bağımlılıklar, derlenen wrapper çıktısı ve Java sonuçları `.data/` içinde kalır.

Çalıştırıcıda ayrı işlemler bulunmalı:

1. **Ön kontrol:** JDK, referans kimliği ve gereken dosyaları doğrula. Her test ailesi yalnızca ihtiyaç duyduğu girdileri zorunlu tutsun; VarInt testi için GPU/asset gerektirme.
2. **Referans yakalama:** Özgün senaryo girdisini Java'ya ver, sürüm/ortamla birlikte çıktıyı yerelde kaydet. Aynı senaryoyu tekrar çalıştırıp kararlılığı kontrol et. Bu işlemin başarılı olması Rust eşitliğinin geçtiği anlamına gelmez.
3. **Karşılaştırma:** Aynı girdiyi Rust adaptörüne ver; çıktıları Java referansıyla karşılaştır. Eksik adaptör, eksik golden veya yanlış sürüm başarı üretemez. Eşitlik komutu çalıştırılmamış zorunlu senaryoları başarılı çıkışla gizlememeli.
4. **Fark raporu:** İlk ayrışan olay/alanı, beklenen ve gerçekleşen değeri, referans kimliğini ve aynı senaryoyu yeniden çalıştırma bilgisini `.data/runs/` altına yaz.

Dil bağımsız senaryo/sonuç kaydı şu bilgileri taşımalı; mevcut uygulama JSONL kullanır; operasyon alanları [çalıştırıcı belgesinde](behavior-runner.md) tanımlıdır:

| Alan | Anlam |
| --- | --- |
| `schema_version`, `case_id`, `operation` | Kayıt şeması ve sabit davranış kimliği |
| `reference_id` | Client JAR, ilgili kaynak/asset seti, JDK kimliği |
| `initial_state`, `inputs` | Başlangıç durumu ve sıralı girdiler |
| `clock`, `seed`, `environment` | Gereken senaryolarda kontrollü koşullar |
| `events`, `outputs` | Sırası korunmuş gözlemler ve son durum |
| `outcome` | Değer/normal dönüş veya gözlenen hata |

64 bit tamsayılar ve float değerler taşıma formatında hassasiyet kaybetmemeli. `float`/`double` için ham IEEE bit gösterimi kullanılmalı; `NaN`, sonsuzluk ve işaretli sıfır JSON sayısına zorlanmamalı. İnsan tarafından okunabilir ondalık gösterim yardımcı olabilir, karşılaştırmanın yerini alamaz. Koleksiyon sırası davranışsa normalize edilmemeli.

P00'ın gerekli doğrulamaları: sabit senaryonun tekrarı; kasıtlı değiştirilmiş tek alanın yakalanması; eksik/yanlış referansın reddi; Rust yokken eşitlik iddiası üretilmemesi. Bu birkaç kontrol dışında genel test framework'ü için kapsamlı test yazımı yapılmaz. Golden yenileme ayrı ve açık işlem olmalı; normal test koşusu beklenen sonuçların üstüne yazmamalı.

## P01.a — Kullanılan sayısal işlemler ve Vec2

Kaynak: `net/minecraft/world/phys/Vec2.java` ve `net/minecraft/util/Mth.java`. Başlangıçta `Vec2.normalized/scale/length/lengthSquared` ile sonraki dilimlerde gereken `floor/clamp/wrapDegrees` overload'ları ele alınacak. İlgisiz tüm yardımcılar topluca taşınmayacak.

| Davranış ailesi | Girdiler | Gözlem ve kabul |
| --- | --- | --- |
| V2-NORMALIZE | Sıfır, her eksendeki iki yön, dört diyagonal, küçük/çok küçük ve büyük uzunluklar | İki bileşenin ham float bitleri, gerekirse uzunluk; Java ile kesin eşitlik |
| V2-OPERATIONS | Sıfır/negatif/pozitif çarpan, farklı büyüklükler; normalizasyonla ardışık uygulama | Her işlem sonrası bileşen/uzunluk bitleri; işlem sırası korunur |
| MTH-FLOOR | Pozitif/negatif tamsayılar ve hemen alt/üst komşuları | Aynı overload'dan tamsayı sonuç; özel değerlerin kapsamı Java çağrısıyla belirlenir |
| MTH-ANGLE | Sarma sınırları, hemen komşuları, çok turlu açılar ve işaretli sıfır | Java overload'ıyla aynı değer/bit sonucu |
| MTH-CLAMP | Aralık içi/dışı ve sınır değerleri; çağrı yerlerinde geçerli aralıklar | Aynı değer ve float bitleri; ters aralık davranışı ancak çağrı kapsamı gerektiriyorsa ayrı |

Test tablosu, Rust algoritmasını tekrar yazan hesaplamalarla beklenen değer üretmez. Özgün girdiler depoya girebilir; Java çıktı dosyaları yerelde kalır. Float→double yükseltme veya farklı işlem gruplama nedeniyle oluşan küçük fark otomatik toleransla kabul edilmez.

## P02.a — DeltaTracker.Timer

Kaynak: `net/minecraft/client/DeltaTracker.java`, `Timer.advanceGameTime/advanceRealTime/updatePauseState/updateFrozenState` ve üç getter. Adaptör, başlangıç zamanı ile `targetMsptProvider` girdisini kontrol edebilir; gerçek sistem saatini beklemesi gerekmez.

| Davranış ailesi | Sıralı senaryo | Gözlem ve kabul |
| --- | --- | --- |
| TIME-BOUNDARY | Aynı başlangıçtan 0, 1, 49, 50, 51 ms ve ardışık küçük artışlar | Dönen tick sayısı, delta ve artık tick bitleri |
| TIME-RESIDUAL | Birikmiş kesirli zamanı birkaç çağrıda tam tick'e taşıma | Her çağrı sonrası çıktı; yalnızca toplam tick karşılaştırması yeterli değil |
| TIME-RATE | Kontrollü sağlayıcıyla farklı hedef tick süreleri ve süre değişimi | Oyun zamanı ile gerçek zaman çıktılarının ayrı eşitliği |
| TIME-PAUSE | Artık zaman varken pause, tekrarlı pause, ilerletme, unpause | Pause öncesi saklanan artık ve getter sonuçları; çağrı sırası korunur |
| TIME-FREEZE | Freeze aç/kapat; getter'a iki farklı `ignoreFrozenGame` değeri | Aynı frozen/paused kombinasyonunda Java çıktıları |
| TIME-REAL | Gerçek zaman delta sınırının altı/eşiti/üstü ve uzun frame | Gerçek zaman getter'ının özel dalı dahil bit eşitliği |

Timer testinin büyük zaman artışında ürettiği tick sayısına, `Minecraft.runTick` içindeki 10 tick yürütme sınırı uygulanmaz. Bu sınır ayrı P02.b döngü testidir. Ayrıca zamanlayıcının pause API'si, multiplayer dünyasının gerçekten durduğu anlamına gelmez.

## P03.a — VarInt ve VarLong

Kaynak: `net/minecraft/network/VarInt.java` ve `VarLong.java`. Java adaptörü gerçek Netty buffer üzerinden `getByteSize`, `write`, `read` çağırır. Codec algoritmasını adaptörde yeniden yazmaz.

| Davranış ailesi | Girdiler | Gözlem ve kabul |
| --- | --- | --- |
| VAR-WRITE | 0, 1, -1, işaretli tür min/max ve her 7 bit uzunluk sınırının komşuları | Java'nın yazdığı baytlar ve `getByteSize` sonucu |
| VAR-READ | Java'nın yazdığı geçerli diziler ve bağımsız sentetik bayt girdileri | Okunan değer, tüketilen bayt sayısı ve kalan veri |
| VAR-TRUNCATED | Boş buffer, tamamlanmayan devam bitleri ve her kesme konumu | Başarı/hata, Java exception kategorisi ve hataya kadar tüketim |
| VAR-OVERLONG | VarInt için 5, VarLong için 10 bayt sınırının çevresi ve daha uzun devam dizileri | Java'nın kabul/red davranışı ve cursor; daha katı decoder varsayma |
| VAR-NONCANONICAL | Gereksiz uzun temsil, son bayttaki sınır bitleri ve arkasına eklenen veri | Java'nın gerçek sonucu; “geçersiz olmalı” varsayımıyla oracle'ı değiştirme |

Java exception sınıfı referans kaydında tutulur; Rust'ın aynı dil sınıfını üretmesi beklenmez. Rust adaptörü hata kategorisini eşleyecek; protokolü/kullanıcı sonucunu etkileyen hata davranışı ve tüketim korunacak. Java stack trace metni eşitlik ölçütü değildir.

## Sonraki testlerin otomasyon yolu

Bu ilk saf işlemler grafik arayüzsüz doğrudan çağrılabilir. `LocalPlayer`, dünya, GUI ve renderer testleri aynı derecede bağımsız değildir. İlgili bölümde şu sınırlar kullanılacak:

- Dünya/input/packet senaryolarında Java client'ın gerçek mantığı çalıştırılır; başlangıç durumu ve olay girdileri kontrollü kurulur. Test için olay besleyen adaptör üretim sunucusu değildir. Referans mantığını basitleştiren sahte `LocalPlayer` beklenen sonuç kaynağı olamaz.
- Gerekirse özgün bir Java instrumentasyon adaptörü kullanılır; gözlem ve kontrollü girdi sınırları belgelenir. Decompile algoritmayı yeniden yazmak veya oyun sonucunu değiştiren hook kullanmak referansı geçersizleştirir. Uygulama yöntemi henüz seçilmedi.
- Grafik/ses referansı için ayrı oyun dizini, sabit ayarlar ve otomatik input replay hazırlanır. Kullanıcının Prism ayarları/save'leri test alanı yapılmaz. Frame/ses yakalama ve fark raporu kullanıcıdan elle oyunu yönetmesini beklememeli.
- Java ve Rust'ın farklı frame bölünmelerinde kendi aralarında aynı sonucu vermesi peşinen şart koşulmaz; **her kontrollü zaman çizelgesinde Rust, o çizelgedeki Java sonucuyla** karşılaştırılır.
- Donanım/servis erişimi eksikse senaryo engelli kalır. GPU veya ses yakalama henüz kurulmadı diye saf referans testleri bekletilmez. Bu sonraki adaptörlerin çalışabilirliği bu turda doğrulanmış değildir.

İlk çalışma tamamlandığında rapor Java referansının hazır olup olmadığını, hangi Rust davranışlarının mevcut olduğunu ve gerçekten karşılaştırılan senaryoları ayrı göstermeli. “Test altyapısı çalışıyor” ile “oyun Java ile aynı” birbirinin yerine kullanılmaz.
