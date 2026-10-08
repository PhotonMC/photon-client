# P16.p — Blockstate dosyaları ve üç eksenli varyant dönüşü — karışık kanıt kaydı

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

İki kanıt sınıfı taşır:

- `client-parity`: blockstate dosyasının **ayrıştırılması** ve üç eksenli
  varyant dönüşü. İkisi de pinned JAR'daki gerçek kodekslere ve gruba verilip
  karşılaştırılır.
- **Türetilmiş**: bir seçicinin veya koşulun gerçek bir blok durumuyla
  **eşleştirilmesi**. Referansta bu adım bloğun durum tanımını istiyor, o da
  kayıt defterini istiyor; ayrıştırma ise hiçbirini istemiyor.

## Java kaynak incelemesi

Bir blok modelini doğrudan adlandırmaz. Blockstate dosyası ya seçici
dizgelerini modellere bağlar (`variants`), ya koşulu tutan parçaları listeler
(`multipart`), ya da **ikisini birden** taşır — sonuncusu kodekste açıkça
serbest ve bir satırla pinlendi.

Varyantın dönüşü üç eksenlidir: `Variant.SimpleModelState.asModelState`
`fromXYZAngles(x, y, z)` çağırıyor. Bizde yalnızca iki eksen vardı; eksik
olan `z` bu dilimde kapandı. Grup değişmeli olmadığı için sıra serbest
değil — X önce, Y onun dışına, Z ikisinin dışına sarılıyor.

Z üretecinin matrisi tahmin edilmedi, oracle'dan okundu:
`(x, y, z) → (-y, x, z)`.

Ayrıştırmanın gözlenen ayrıntıları:

- Ağırlık `POSITIVE_INT`: sıfır ve negatif reddediliyor, yokluğunda bir.
- Boş `variants` haritası, boş `multipart` listesi ve boş varyant listesi
  reddediliyor; hiçbir şey çizmeyen bir dosya hata sayılıyor.
- `when` içindeki bir değer metin, tamsayı veya mantıksal olabiliyor; son
  ikisi metne çevriliyor ve **kesirli bir sayı tamsayıya kırpılıyor**
  (`3.5` → `"3"`).
- Bir birleştirici tam olarak bir işlem anahtarı taşımalı. Yanında bir özellik
  varsa dosya reddediliyor — özellik sessizce düşürülmüyor. İşlem adı büyük
  harf; `or` reddediliyor.
- Boş bir işlem listesi (`{"AND": []}`) kabul ediliyor.
- Terimdeki boş bir parça (`"north|"`) bütün koşulu reddediyor.
- Seçici dizgesi ayrıştırma sırasında **hiç denetlenmiyor**: `"=,,=nope"`
  kabul ediliyor. Denetim ancak durum tanımıyla karşılaşınca yapılıyor.

## Girdiler

`model/blockstate-parse` otuz dört satır taşır: tek varyant, özellikli seçici,
üç eksenli dönüş ve sarmalanmış açılar, ağırlıklı listeler ve geçersiz
ağırlıklar, eksik ve bozuk model adları, her iki bölümü taşıyan bir dosya, ve
multipart'ın koşul biçimlerinin tamamı — düz, çoklu terim, olumsuzlanmış,
`AND`, `OR`, iç içe, mantıksal ve sayısal değerler, ve yedi ret durumu.

`model/model-state` yüz on beş satıra çıktı: iki eksenli altmış dört satırın
yanına `z`'nin tek başına ve diğer ikisiyle birlikte sürüldüğü elli bir satır
eklendi.

## Sonuçlar

Java ve Rust her satırda aynı yapıyı ve aynı reddi verdi. Dönüş matrisleri ve
altı yüzün dönüşümleri float bit düzeyinde eşleşti.

Koşu: `python3 tools/behavior/run.py capture` ardından
`compare --adapter target/debug/examples/behavior_adapter` → `parity_passed`,
3899 senaryo / 12 aile.

## Sondaya bağlanması

Sonda artık model yolu yerine blok adı alabiliyor:

```
PHOTON_ASSETS=<assets> PHOTON_BLOCK=minecraft:oak_stairs \
PHOTON_STATE="facing=north,half=bottom,shape=straight" cargo run -- window
```

Blockstate dosyası okunuyor, özellikler varyantı seçiyor, model ve dönüş
oradan geliyor. `facing=east` dönüşsüz çıkıyor (modelin kendi yönü),
`facing=north` `y=270` ve `uvlock` istiyor.

Seçilen varyantın UV kilidi de uygulanıyor; bkz. [P16.q](uv-lock-conformance.md).

## Açık kapsam

Eşleştirme pinlenmedi ve referansla iki noktada ayrılıyor:

1. Referans tanımadığı bir özellik adı veya değeri için **hata fırlatıp** o
   seçiciyi atlıyor ve günlüğe yazıyor; bizde bilinmeyen bir özellik yalnızca
   eşleşmiyor.
2. Ağırlıklı bir listeden seçim referansın kendi rastgele kaynağını istiyor;
   sonda listenin ilkini alıyor.

İkisi de blok kayıt defteri geldiğinde kapanmalı.

Çok parçalı bir blok (çit, duvar, kızıl taş teli) komşularına bakan özellikler
taşıyor; sonda onları doğrudan veriyor, dünyadan türetmiyor.
