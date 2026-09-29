# P10.h — LocalPlayer tick bileşimi — client-parity kayıt

**Tarih:** 2026-09-27 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity`

## Ne pinlendi

Hareketin yaprakları (input çarpanları, sürtünme, travel dalları, çarpışma
çözümü, basamak adayları) daha önce tek tek pinliydi. Bu kayıt onları
birleştiren zinciri pinliyor: istemcinin kendi oyuncusunun bir tick'i,
referansın çalıştırdığı sırayla, baştan sona.

Oracle gerçek bir `LocalPlayer` kuruyor ([fixture kaydı](movement-fixture-record.md)):
`KeyboardInput`, gerçek `ClientLevel`, gerçek oyun modu nesnesi, paketleri
yakalayan bir bağlantı. Her tick `ClientLevel.tickNonPassenger` ile sürülüyor,
yani eski konumun kaydı ve tick sayacı da referansın sırasında.

Rust tarafı `crates/voxel-world/src/local_player.rs`: seviyenin eski-konum adımı,
`Player.tick`, sıvı etkileşimi güncellemesi ve yüzme bayrağı,
`LocalPlayer.aiStep` (eğilme, sprint penceresi, yaratıcı çift zıplama,
izleyici uçuşu, suda batma, uçarken dikey girdi), `LivingEntity.aiStep`
(küçük hızların sıfırlanması, zıplama ve sıvıda zıplama), `Player.travel`
(yüzme bakış düzeltmesi, uçuşta dikey hızın korunması), hava/su/lav travel,
`Entity.move` (kenardan geri çekilme, çarpışma ve basamak çıkma, zemin ve
destek bloğu, düşme mesafesi ve onu sıfırlayan ışın testi, sekme, blok hız
çarpanı), otomatik zıplama ileri bakışı, blok etkileri (balçığın basış
etkisi, içinden geçilen blokların gezinimi ve bal kayması), poz güncellemesi
ve tick'in gönderdiği paketler.

## Senaryolar

`tools/behavior/cases.py` içindeki `movement_cases`. Her senaryo bir sahne,
başlangıç pozu ve her tick basılı tuşlar. Her tick için karşılaştırılan:
konum ve hız (f64 bitleri), bakış (f32 bitleri), zemin/yatay/dikey/küçük yatay
çarpışma bayrakları, sprint, shift, poz, düşme mesafesi, uçuş, ve gönderilen
her paketin türü ile kodlanmış baytları.

Kapsanan dallar: düz zeminde her yön, dönüş, durma, sprint ve bırakma,
eğilerek yürüme, tek ve basılı zıplama, sprint zıplaması, duvar, yarım blok
ve tam blok basamak, bloğa zıplama, düşüş, uzun düşüş, kenardan düşme ve
eğilerek kenarda durma, buz, ruh kumu, balçığa düşüş ve eğilerek balçığa
düşüş, bal, merdiven, tuzak kapı merdiveni, kenardan merdivene düşüş, yüzme,
koşarak yüzme, suda yükselme ve batma, suya yüksekten düşüş, akan su, duvar
önündeki şelalede yüzme (düşen sıvının aşağı çekişi), çit yanından akan su
(tam blok olmayan komşunun `blocksMotion`'ı), lav,
tavan altında sürünme pozu, tavana zıplama, duvar boyunca sprint (küçük yatay
çarpışma), bal duvarında kayma, otomatik zıplama ve sprintle otomatik
zıplama, aşağı bakarak yürüme, yaratıcı çift zıplama ile uçuş, uçarken sprint
ve inişte uçuşun kapanması, izleyici uçuşu, toz kara düşüp içinde yürüme
ve zıplama, zeminde toz kara girme, örümcek ağı, tatlı meyve çalısı,
yukarı iten kabarcık sütununda yükselme, aşağı çeken sütunda batma,
bitkilerin (çim, eğrelti, çiçek, fidan, tatlı meyve çalısı, uzun çim)
içinde eğilip kalkma.

Her tick ayrıca oyuncu modelinin poz girdileri karşılaştırılıyor (yarım
karede): gövde ve baş yönü (`tickHeadTurn`, yürüme yönüne dönme, başa en
fazla 50°), yürüme animasyonunun konumu ve hızı, yüzme pozuna geçiş,
bakış sallanmasının yürünen mesafesi ve sallanması. Referans yeni bir
canlıyı rastgele bir yöne çevirerek kuruyor; oracle ve Rust baş ile
gövdeyi başlangıç bakışına eşitleyerek başlıyor.

İçine girilen bloklar referanstaki sırayla geziliyor: hareketin başındaki
kutu, yol boyunca köşenin geçtiği hücreler ve hedef kutu, her biri hareket
yönünün köşesinden başlayıp eksenleri adım sırasıyla dolaşarak. Kabarcık
sütununda her hücre hızı ayrı ayrı sınırladığı için bu sıra sonucu
değiştiriyor. Toz kar oyuncuyu ancak 2,5 bloktan fazla düşünce tutuyor
(0,9 yüksekliğinde zemin); deri çizme takılı sayılmıyor, çünkü istemci
ekipmanı henüz izlemiyor.

## Sonuç

`python3 tools/behavior/run.py compare --family movement` → `parity_passed`.
Bütün senaryolar her tick'te bit bit eşleşiyor.

## Taşınmamış dallar

Taşınmamış bir dala girilince tick yaklaşık değer üretmiyor, `Unsupported`
döndürüyor:

- Çarpışma şekli varlığa bağlı iskele.
- Dünyanın altına düşme.

Toz karın donma etkisi (`isInPowderSnow`) ve sunucunun donmaya göre
verdiği hız cezası tutulmuyor. İlki yalnızca deri çizmeyle tırmanmayı,
ikincisi sunucunun gönderdiği özellikleri etkiliyor; istemci özellik
paketlerini de henüz uygulamıyor.

Hiç yazılmamış olanlar (senaryoya da girmiyorlar): elytra ile süzülme, eşya
kullanırken yavaşlama, iksir etkileri (hız, zıplama, yavaş düşüş,
levitasyon, körlük), açlık nedeniyle sprintin kapanması, araçlar, başka
varlıklarla çarpışma ve itilme, dünya sınırı çarpışması.

## Tablolar

Blok başına sürtünme, hız ve zıplama çarpanı, sekme ve etiket/sınıf bayrakları
ile durum başına sıvı ve boğma bilgisi referanstan yakalanıp
`crates/voxel-data/src/block/movement.rs` olarak üretiliyor; block ailesi her koşuda
tabloyu baştan sona karşılaştırıyor. Sıvı akışının okuduğu durum özellikleri
(tam sağlam yüzler, buz blokları hariç; `blocksMotion`) da aynı yolla
`crates/voxel-data/src/block/fluid_flow.rs` (`tools/generate/fluid_flow.py`). Üretim:

```
python3 tools/generate/block_movement.py .data/captures/behavior/<id>/outputs.jsonl
```
