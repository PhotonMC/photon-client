# P16.d — Strict JSON ayrıştırma ve sayı dönüşümü — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik metin pinned JAR'ın
kullandığı Gson yapılandırmasına ve üretim kütüphanesindeki
`photon::json` modülüne verilir, kabul/ret kararı ile ayrıştırılan
belgenin kanonik hâli birebir karşılaştırılır.

Render veri katmanının dördüncü dilimi. Model, blockstate ve pack dosyalarının
okunabilmesi buna bağlıdır.

## Bağımlılık kararı

`serde_json` depoda yalnızca geliştirme bağımlılığıdır ve
[AGENTS.md](../../AGENTS.md) onu açıkça "uygulama stack'i seçimi değildir" diye
işaretler. Ayrıştırıcı bu yüzden depoda yazıldı; kullanıcıya sorulmadan o kural
bozulmadı. Teknik gerekçe de aynı yönde: eşleşmesi gereken şey genel bir JSON
kütüphanesinin davranışı değil, referansın yapılandırdığı Gson'un davranışıdır.
[PNG çözücüde](png-decode-conformance.md) aynı yaklaşım gerçek bir boşluk
yakalamıştı.

## Java kaynak incelemesi

`GsonHelper`, okuyucuyu `Strictness.STRICT` ile kurar. Gson'un başka yerlerde
izin verdiği gevşeklikler burada rettir: yorum, tek tırnak, tırnaksız anahtar,
sondaki virgül, `NaN`, `Infinity`, `+1`, `01`, `0x10`, `.5`, `1.`.

Ayrıştırılan belge hakkında iki davranış temsili belirler:

- **Sayılar literal olarak saklanır.** Gson sayıyı çağıran bir tip isteyene
  kadar dönüştürmez; `1E3`, `1e+3` ve `1e400` yazıldığı gibi durur.
- **Dizgiler UTF-16'dır.** `\ud83d` gibi eşleşmemiş bir surrogate kabul edilir
  ve saklanır. Rust `String`'i bunu tutamayacağı için modül dizgileri UTF-16
  kod birimi olarak taşır.

Nesne alanları eklenme sırasını korur ve tekrar eden anahtar **yerini korur**:
`{"a":1,"b":2,"a":3}` sonucu `a` ilk sırada, değeri `3` olacak şekilde çıkar.
Anahtarı sona taşıyan bir implementasyon belgeyi yeniden sıralardı.

## Girdiler

`json/parse` 50 parametrik satır taşır. 26'sı kabul edilmeli: boş nesne ve
dizi, iç içe yapı, gerçek model biçimi, boşluk varyantları, anahtar sırası,
tekrar eden anahtar ve konumu, tam sayı/ondalık/üstel/çok büyük sayı
literalleri, negatif sıfır, yirmi seviye derinlik, unicode kaçışları, surrogate
çifti, eşleşmemiş surrogate, bütün kaçış dizileri, ham UTF-8, boş anahtar ve
üst seviye skaler değerler.

24'ü reddedilmeli: iki tür sondaki virgül, tek tırnak, tırnaksız anahtar, iki
tür yorum, `NaN`, `Infinity`, başta artı, başta sıfır, onaltılık sayı, başta ve
sonda ondalık nokta, kapanmamış dizgi ve nesne, geçersiz kaçış, dizgi içinde
ham kontrol karakteri, belgeden sonra içerik, boş girdi, yalnızca boşluk, eksik
iki nokta, eksik virgül ve çift virgül.

## Sonuçlar

Java ve Rust her satırda aynı kabul/ret kararını ve kabul edilenlerde aynı
kanonik hâli verdi.

Koşu: `python3 tools/behavior/run.py capture --family json` ardından
`compare --family json --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

### Gerçek dosyalarla doğrulama

Sentetik kapsam geçtikten sonra iki yerel kontrol yapıldı (izlenmeyen):

- Pinned JAR içindeki **14.163 JSON dosyasının tamamı** ayrıştırıldı, sıfır ret.
- Rastgele seçilen **400 gerçek dosya** hem Java oracle'ına hem Rust'a verildi;
  kanonik çıktılar **400/400 birebir** eşleşti.

## Sayı dönüşümü

`json/number` 61 literali dört tipe çevirip karşılaştırır. Sonuçlar tahmin
edilebilir değildir ve hiçbiri tesadüf değildir:

| Literal | `int` | `long` | `float` |
| --- | ---: | ---: | --- |
| `2147483648` | `-2147483648` | `2147483648` | `2147483648.0` |
| `9223372036854775807` | `-1` | `9223372036854775807` | — |
| `9223372036854775808` | `0` | `-9223372036854775808` | — |
| `1e38` | `0` | `687399551400673280` | `9.99999968e37` |
| `1.5` | `1` | `1` | `1.5` |
| `-2.5` | `-2` | `-2` | `-2.5` |
| `16777217` | `16777217` | `16777217` | `16777216.0` |
| `3.4028235e38` | `-2147483648` | — | son sonlu float |
| `3.4028236e38` | `0` | — | `inf` |
| `1e-46` | `0` | `0` | `0.0` |

Referans tam sayıya üç ayrı yoldan varır — doğrudan `int` ayrıştırma, `long`
ayrıştırıp daraltma, ya da büyük ondalıktan düşük bitleri alma — ve üçü de aynı
kurala indirgenir: **sıfıra doğru kırp, düşük 32 (veya 64) biti al.** Taşan bir
literal doygunlaşmaz, sarar. Modül bunu basamak dizisi üzerinde modüler
aritmetikle yapar; büyük ondalık sayı tipi gerektirmez.

Kayan noktada davranış ayrı: taşma sonsuza gider, `float` aralığının altı
sıfıra iner ve `16777217` gibi bir tam sayı en yakın temsil edilebilir değere
yuvarlanır.

## Açık kapsam

`GsonHelper`'ın alan düzeyindeki yardımcıları (`getAsInt(object, name)`, eksik
alan varsayılanları, tip uyuşmazlığında üretilen hata metinleri) bu kaydın
dışındadır. Geçersiz UTF-8 baytlarının davranışı da sınanmadı: referans metni
dosyadan zaten çözülmüş alır, bu yüzden çözme sınırı bu kaydın dışındadır.
