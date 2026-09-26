# P16.h — Quad normali, yön seçimi ve winding — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün dört köşe pinned JAR'daki
`GeometryUtils.normal`, `FaceBakery.findClosestDirection` ve
`FaceBakery.recalculateWinding` yoluna ve üretim kütüphanesindeki
`voxel_client::render::geometry` modülüne verilir; normal, seçilen yön, yeniden
sıralanmış köşeler ve taşınan yük sırası karşılaştırılır.

Bu, quad pişirmesinin son adımıdır: element döndürüldükten sonra yüzün hangi
yöne baktığı ve köşelerinin hangi sırada durduğu burada belirlenir.

## Java kaynak incelemesi

Normal, ilk köşeden çıkan iki kenarın çapraz çarpımıdır ve ardından
normalleştirilir. Bileşen sırası korunmalıdır çünkü sıfırın işaretini o
belirler: referans-sıralı bir north yüzünde normalin `y` bileşeni **negatif
sıfır** çıkar. Normalleştirme ters karekökü çift duyarlıkta alıp tek duyarlıkta
bölür. Dejenere bir üçgen sıfıra bölünür ve `NaN` üretir; bu hata değildir,
sonraki adım onu yakalar.

`findClosestDirection` non-finite bir normalde hiçbir yön döndürmez. Aksi halde
altı yönün nokta çarpımları arasında **kesin pozitif** olup en büyüğü kazanır.
Başlangıç eşiği sıfır olduğu için tam kenarına bakan bir yüz hiçbir eksene
sahip çıkmaz ve eşitlikte önce gelen yön kalır.

`recalculateWinding` dört köşeyi verilen yönün beklediği sıraya geri koyar.
Quad'ın kendi sınırlayıcı kutusunu `999`/`-999` tohumlarıyla bulur, her yuva
için o kutunun hangi köşesini istediğini hesaplar ve eşleşen köşeyi yerine
takas eder. Eksen hizalı olmayan bir quad'ın köşeleri o kutunun köşeleri
olmadığı için eşleşme bulunmaz ve referans tahmin etmek yerine **istisna
atar**.

## Girdiler

`model/quad-winding` 12 satır taşır: altı yönün referans-sıralı tam blok
yüzleri, ters sarılmış bir yüz, karıştırılmış köşeler, kısmi bir kutu, dört
köşesi aynı olan dejenere girdi, eşdoğrusal köşeler ve köşegen bir quad.

## Sonuçlar

Java ve Rust her satırda aynı sonucu verdi:

| Girdi | Normal | Yön | Sonuç |
| --- | --- | --- | --- |
| `up-ordered` | `(0, +1, 0)` | `up` | sıra değişmedi |
| `north-ordered` | `(0, -0, -1)` | `north` | sıra değişmedi, `y` negatif sıfır |
| `up-reversed` | `(0, -1, 0)` | `down` | yük sırası `30, 40, 10, 20` |
| `up-shuffled` | `(0, -1, -0)` | `down` | yük sırası `40, 20, 30, 10` |
| `degenerate` | `NaN` | yok | yön seçilmedi |
| `collinear` | `NaN` | yok | yön seçilmedi |
| `diagonal` | `(0, -0.707, +0.707)` | `down` | **eşleşen köşe yok** |

Son satır referansın istisna attığı yolu sabitler; eksen hizalı olmayan bir
quad'ın winding'i onarılamaz.

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

`bakeQuad`'ın tamamı hâlâ açıktır: sprite'a bağlı UV paketlemesi, UV döndürme
çeyrekleri, model durumu dönüşümü ve saydamlık hesabı. Element döndürmesi
[kendi kaydındadır](element-rotation-conformance.md); winding yalnızca
döndürme yokken uygulanır, bu koşul da o kayıtla birlikte değerlendirilir.
