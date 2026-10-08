# P16.j — Model dosyası ayrıştırma — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün model metni pinned JAR'daki
`CuboidModel.fromStream` yoluna ve üretim kütüphanesindeki
`photon::render::model::parse_model` fonksiyonuna verilir; kabul/ret
kararı, parent, ambient occlusion, GUI ışığı, texture slotları ve her elementin
kutusu, döndürme dönüşümü, gölge bayrağı, ışık yayımı ve yüzleri karşılaştırılır.

Bu dilim önceki veri dilimlerini birleştirir: metin
[JSON ayrıştırıcısından](json-parse-conformance.md) geçer, sayılar
[aynı kayıttaki dönüşümle](json-parse-conformance.md) okunur, döndürme
[element döndürme kaydındaki](element-rotation-conformance.md) matrislerle
kurulur ve yüz döndürmesi
[UV kaydındaki](face-uv-rotation-conformance.md) çeyrekleri kullanır.

## Java kaynak incelemesi

Model dosyası, kendi deserializer'ları kayıtlı bir okuyucudan geçer. Ayrıştırma
sırası şudur: elementler, parent adı, texture haritası, ambient occlusion,
display dönüşümleri, GUI ışığı.

Reddedilme koşulları tek tek yazılıdır ve hiçbiri sessizce toparlanmaz:

- `from` ve `to` üç değer taşımalı ve `-16` ile `32` arasında kalmalıdır.
- Bir elementin en az bir yüzü olmalıdır; tanınmayan bir yön adı hatadır.
- `shade` varsa **boolean olmalıdır**; sayı kabul edilmez. Referans dönüştürmeye
  girmeden önce tipi denetler, yoksa `1` değeri `false`'a çevrilirdi.
- `light_emission` varsa sayı ve `0`–`15` aralığında olmalıdır.
- `rotation` bloğu ya `axis` ve `angle` ya da `x`/`y`/`z` taşımalıdır; ikisi de
  yoksa hatadır. `axis` tanınmayan bir eksen adıysa hatadır.
- Yüzün `uv` alanı varsa tam dört değer taşımalıdır.
- Yüzün `rotation` alanı doksanın katı olmalıdır.
- Yüz bir `texture` adlandırmalıdır.

Sessizce toparlanan iki durum vardır ve ikisi de kayda geçirildi: tanınmayan
bir `cullface` adı hata değil, cullface yokluğudur; tanınmayan bir `gui_light`
adı da alanı ayarsız bırakır.

`rotation` bloğunun `origin`'i on altıncılarla yazılır ve `0.0625` ile
çarpılarak blok birimine çevrilir.

## Girdiler

`model/parse-model` 28 satır taşır. On üçü kabul edilmeli: boş nesne, yalnızca
parent taşıyan gerçek `cube_all` biçimi, altı yüzlü tam küp, ambient occlusion
ve GUI ışığı geçersiz kılmaları, kendi UV'si tint ve döndürmesi olan yüz, iki
döndürme yazımı, `shade:false`, `light_emission:15`, sınırlara tam dayanan
kutu, üç biçimi karışık texture haritası ve iki elementli model.

On beşi reddedilmeli: sınır dışı iki kutu, kısa vektör, yüzsüz element,
tanınmayan yön, boolean olmayan `shade`, aralık dışı iki `light_emission`,
boş döndürme bloğu, geçersiz eksen, kısa `uv`, doksanın katı olmayan yüz
döndürmesi, texture'suz yüz, nesne olmayan üst seviye ve sondaki virgül.

## Sonuçlar

Java ve Rust her satırda aynı kararı ve kabul edilenlerde aynı yapıyı verdi.

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

### Gerçek dosyalarla doğrulama

İki yerel kontrol (izlenmeyen):

- Pinned JAR içindeki **3.928 model dosyasının tamamı** ayrıştırıldı, sıfır ret.
- Rastgele **400 gerçek model** hem Java oracle'ına hem Rust'a verildi; çıktılar
  **400/400 birebir** eşleşti.

## Açık kapsam

`display` dönüşümleri ayrıştırılmıyor; referans onları `ItemTransforms` ile
okur ve bu ayrı bir dilimdir. Parent zincirinin dosyalardan kurulması, yani bir
model adından başlayıp kaynak kökünden okuyarak zinciri yürümek de bu kaydın
dışındadır; bu kayıt tek bir dosyanın ayrıştırılmasıdır. Blockstate varyant
seçimi ve model durumu dönüşümü de açıktır.
