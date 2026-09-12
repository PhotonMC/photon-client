# 2026-09-12 yön düzeltmesi: kanıt sınıfı, varyant patlaması ve kritik yol

Bu belge bir davranış dilimi kaydı değildir; deponun 2026-09-07 → 2026-09-12
arasındaki gidişinin incelenmesi ve buradan çıkan bağlayıcı kurallardır. Yeni
bir dilime başlamadan önce [AGENTS.md](../AGENTS.md), [roadmap](roadmap.md) ve
bu belge birlikte okunur.

## İncelemenin dayandığı ölçümler

| Ölçüm | Değer |
| --- | ---: |
| Commit | 401 |
| Rust satırı | 21.506 |
| `src/connection.rs` satırı | 9.835 |
| `connect_and_probe_with_mode` gövdesi | ~2.500 satır |
| `PlayProbeMode` varyantı | 84 |
| `docs/` dosyası | 741 |
| `docs/` satırı | 23.167 (kod satırından fazla) |
| contract / parity belgesi | 366 / 367 |
| Rust unit testi | 99 |
| Çalıştırılabilir istemci yüzeyi | yok (`main.rs` boş) |

Commit tipi dağılımı yön değişimini gösterir: 2026-09-09/10 tarihlerinde 250
`feat:` commit'i implementasyon üretti; 2026-09-11/12 tarihlerindeki 89
commit'in tamamı `test:` oldu ve bunların ~68'i aynı kod yolunun farklı sabitle
tekrarıydı.

## Korunacak olanlar

Aşağıdakiler doğru kurulmuştur ve sadeleştirme sırasında bozulmamalıdır.

- `tools/behavior` runner'ının `preflight/capture/verify/compare` ayrımı,
  `failed` (1) ile `blocked` (2) çıkış kodlarının ayrılması, referans kimliğinin
  JAR hash'i, JDK sürümü ve senaryo girdisini kapsaması, baseline'ı sessizce
  kabul eden bir koşunun bulunmaması.
- P01–P04 ailelerinin (`wire`, `math`, `timer`, `utf8`, `binary`, `nbt`,
  `resources`) doğrudan pinned JAR sınıflarını çağırarak karşılaştırılması. Bu
  aileler gerçek client parity kanıtı taşır.
- Bağımlılık disiplini (tek üretim bağımlılığı `flate2`), `.data/` ayrımı,
  `Cargo include` desenlerinin depo köküne sabitlenmesi.

## Bulgu 1 — İki farklı kanıt sınıfı tek ad altında toplanmış

P12 ve P13 probe'ları Java **istemcisiyle** karşılaştırma yapmaz. Yaptıkları
şey, Rust'ın ürettiği paketi canlı vanilla **sunucusuna** yollamak ve düzeltme
paketi gözlenmediğinde senaryoyu geçmiş saymaktır. Java oracle bu akışta yalnız
sabit üretir (component type ID, CRC32C hash). Bu, wire uyumluluğu kanıtıdır;
istemci davranış eşitliği kanıtı değildir.

[Davranış testi sözleşmesi](behavior-testing.md) bunu zaten söylüyordu; eksik
olan, kaydın adında ve roadmap satırında bu ayrımın görünmemesiydi. Kanıt
sınıfları artık [behavior-testing.md](behavior-testing.md) içinde tanımlıdır ve
her contract/parity belgesi sınıfını başlıkta bildirir.

## Bulgu 2 — Varyant patlaması

68 belge ve 68 probe, tek bir kod yolunun sabit değiştirilmiş kopyasıdır. Somut
kanıt:

- `sheep/color`, `cat/collar`, `wolf/collar`, `tropical_fish/base_color`,
  `tropical_fish/pattern_color`, `dye`, `dyed_color` component'lerinin yedisi de
  aynı `0x8c6452aa` hash'ini üretir; hepsi `DyeColor.RED` (14) taşır.
- Parser tarafında yedisinin de gövdesi `read_varint` çağrısından ibarettir.
- Parity tablolarının sonuç sütunları birebir aynıdır
  (`click sonrası slot/cursor 0/0`, `ek full content 0`, `chunk 157/157/51`).

Bu, [AGENTS.md](../AGENTS.md) içindeki "implementasyonu tekrar eden test ekleme"
kuralının ve roadmap P24'teki "bir milyon kopya test yerine kaynak dalına ve
gözlenen farka göre genişlet" kuralının ihlalidir. Kural vardı; operasyonel
karşılığı yoktu. Karşılığı artık AGENTS.md'de "Varyant değil dal" başlığıdır.

## Bulgu 3 — `connection.rs` istemci değil, test kaydedicisidir

`PlayConnectionTrace` yalnızca sayaç ve `*_observations` vektörlerinden oluşur.
Depoda `ClientLevel`, chunk deposu, entity deposu veya container menu/slot
modeli yoktur. Chunk paketi çözülür, `ChunkPacketObservation` içindeki uzunluk
ve sayaçlara yazılır ve atılır. Dolayısıyla "P08 ilk chunk teslimi geçti" kaydı
wire tüketimi kanıtıdır, dünya kopyası kanıtı değildir.

Yapısal sonuç: her yeni senaryo bir enum varyantı, beş-altı `match` kolu, bir
public sarmalayıcı ve bir example dosyası ekler. Bakım maliyeti senaryo sayısıyla
doğrusal büyürken kanıt getirisi sabit kalır. Gerçek istemci yazıldığında bu
dosyanın büyük kısmı yeniden yazılacaktır.

## Bulgu 4 — Belge enflasyonu

Her probe commit'i beş ortak dosyaya dokunuyordu: `README.md`, `roadmap.md`,
`remaining-test-plan.md`, `source-review.md` ve iki yeni belge. Sonuçta
`roadmap.md` ve `remaining-test-plan.md` içindeki P12/P13 hücreleri elli maddeyi
aşan tek cümleye dönüştü ve okunamaz hale geldi. Bu hücreler zaten roadmap'in
kendi kuralını ihlal ediyordu: dilim durumu kendi contract/parity belgesinde
tutulur, genel belgelerde hareketli toplam tekrar edilmez.

Bu belgeyle birlikte ilgili hücreler özet biçime indirildi; senaryo listesi
kendi parity belgelerinde durmaya devam eder.

## Bulgu 5 — Kapsam kritik yolda ilerlemiyor

| Bölüm | Gerçek durum |
| --- | --- |
| P00–P03 | Java oracle ile client parity kanıtı var |
| P04–P05 | Kısmi; istemciye bağlanmadı |
| P06–P08 | Wire seviyesinde tüketim var, istemci state modeli yok |
| P12–P13 | Sunucu-uyum probe'u seviyesinde |
| P09, P10, P11 | Başlanmadı |
| P14–P26 | Başlanmadı (kamera, render, animasyon, partikül, ses, font, GUI, chat, dağıtım) |

Başlanmamış bölümler işin hacimce büyük kısmıdır ve en zor parçaları içerir.
Yatay varyant üretimi bu tabloyu hiç değiştirmez. Tek gerçek darboğaz istemci
state modelidir: P09, P10 ve P16 onsuz başlayamaz.

## Bağlayıcı eylemler

Sıradaki ajan bu maddeleri sırayla uygular; her madde ayrı commit'tir.

1. **Component varyant serisi durduruldu.** Yeni bir item component probe'u
   yalnızca [AGENTS.md](../AGENTS.md) "Varyant değil dal" testini geçerse açılır.
   Mevcut 68 kayıt tek bir "item component codec conformance" belgesinde
   birleştirilir; kod tarafında `PlayProbeMode` varyantları yerine
   `(component_yolu, type_id, codec, hash)` tablosu kullanılır. Yeni component
   eklemek tabloya bir satır olmalıdır.
2. **`src/connection.rs` bölünür.** `protocol` (codec ve paket tipleri),
   `session` (durum makinesi) ve probe scaffolding ayrılır; scaffolding
   kütüphaneden çıkar. `PlayProbeMode` enum'u senaryo yapılandırma struct'ına
   çevrilir; davranış enum kolları yerine veriyle ifade edilir.
3. **Kritik yol dilimi açılır: P08.a/b istemci dünya kopyası.** `ClientLevel`
   eşdeğeri, chunk deposu, palette → blockstate çözümü ve section cache. Kabul
   ölçütü, chunk paketinin sayaçlara değil sorgulanabilir bir dünya durumuna
   yazılmasıdır.
4. **Belge protokolü sadeleşir.** Contract ve parity tek dosyada birleşir;
   `README.md` içindeki düz bağlantı listesi üretilen bir dizine taşınır;
   `roadmap.md` ve `remaining-test-plan.md` hücreleri "N senaryo geçti, detay:
   ilgili belge" biçiminde kalır.
5. **`src/main.rs` ölçülebilir bir uç nokta alır:** bağlan, dünya durumunu kur,
   tick döngüsünü çalıştır. Hiçbir şey çizmese de istemci yüzeyi ölçülebilir
   olmalıdır.

## İlerleme ölçütü

Geçen probe sayısı ilerleme ölçütü değildir. Ölçüt, kritik yolda kapatılan
bölüm ve çalıştırılabilir istemci yüzeyidir. Aynı kod yolunun yeni sabit
varyantı ilerleme olarak raporlanmaz.
