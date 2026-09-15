# P05.d.1 — Window GUI ölçeği ve ölçekli boyut — [client-parity] birleşik kayıt

**Tarih:** 2026-09-15 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik girdi hem pinned
JAR'daki `com.mojang.blaze3d.platform.Window` sınıfına hem de üretim
kütüphanesindeki `voxel_client::window` modülüne verilir ve çıktılar birebir
karşılaştırılır. Canlı sunucu bu kayda karışmaz.

Bu, render bölümünün ilk dilimidir ve GPU gerektirmez. GUI ve HUD'daki her
koordinat bu iki davranıştan türediği için, bir yüzey oluşturulmadan önce
sabitlenir. Seçilen render yığınının gerekçesi
[ayrı karar kaydındadır](render-backend-decision.md).

## Java kaynak incelemesi

`com/mojang/blaze3d/platform/Window.java`, `BASE_WIDTH` ve `BASE_HEIGHT`
sabitlerini `320` ve `240` olarak tanımlar.

`calculateScale(maxScale, enforceUnicode)`, ölçeği `1`'den başlatır ve şu dört
koşul birlikte sağlandığı sürece bir artırır: sayaç cap'e eşit değil,
framebuffer genişliği ve yüksekliği sayaçtan büyük, ve bir sonraki ölçekte
tamsayı bölümü her iki eksende de base alanı altına düşmüyor. Cap `0`
verildiğinde sayaç ona hiç eşit olmaz; referans "sığan en büyüğü seç"i böyle
ifade eder. Döngüden sonra unicode zorunluysa ve sonuç tekse bir artırılır —
bu artış cap'i aşabilir.

`setGuiScale(guiScale)`, ölçeği `double`'a yükseltir, framebuffer boyutunu bu
double ile böler, sonucu sıfıra doğru daraltır ve bölüm kalan bırakmışsa bire
ekler. Daraltma Java'nın doyumlu float→int dönüşümüdür; ekleme Java tamsayı
toplaması olduğu için sarar.

## Oracle yöntemi

`calculateScale` ve `setGuiScale` yalnızca ilkel alanları okur/yazar, GLFW
handle'ına dokunmaz. Bu yüzden `WindowOracle`, orijinal sınıfı kurucusunu
çalıştırmadan tahsis eder, `framebufferWidth`/`framebufferHeight` alanlarını
yansımayla yazar ve iki metodu doğrudan çağırır. Native yüklenmez, pencere
açılmaz; çağrılan kod dağıtılan bytecode'dur.

Bu dilimle birlikte `window` ailesi açıldı. Yeni konu, mevcut `math`
catch-all'ına eklenmek yerine kendi ailesini aldı; gerekçe
[yön düzeltmesindedir](course-correction-2026-09-12.md).

## Girdiler

`window/base-size` sabitleri doğrular. `window/gui-scale` 31 parametrik satır
taşır: yaygın framebuffer boyutları, base alanının tam ve bir eksik komşuları,
dejenere boyutlar (`1x1`, `320x2160`, `3840x240`), cap'li ölçekler, unicode
zorlaması ve bölümün kalan bıraktığı boyutlar.

## Sonuçlar

Java ve Rust her satırda aynı `calculated_scale`, `gui_scale`,
`gui_scaled_width` ve `gui_scaled_height` değerlerini verdi. Kaydın ayırt edici
satırları:

| Girdi | Ölçek | Ölçekli boyut | Ne gösteriyor |
| --- | ---: | --- | --- |
| `1920x1080`, cap `0` | `4` | `480x270` | Yükseklik ekseni döngüyü durduruyor |
| `3840x2160`, cap `0` | `9` | `427x240` | Kalan bırakan bölüm yukarı yuvarlanıyor |
| `639x479`, cap `0` | `1` | `639x479` | Base alanının bir altı ölçeği artırmıyor |
| `1920x1080`, cap `1`, unicode | `2` | `960x540` | Unicode artışı cap'i aşıyor |
| `1x1`, cap `0`, unicode | `2` | `1x1` | Sıfır genişlik üretilmiyor; kalan yukarı yuvarlanıyor |
| `1001x701`, cap `7` | `2` | `501x351` | Cap'e ulaşılmadan alan sınırı durduruyor |

Koşu: `python3 tools/behavior/run.py capture --family window` ardından
`compare --family window --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Açık kapsam

Bu kayıt yalnızca ölçek seçimi ve ölçekli boyut yuvarlamasıdır. `Window`'un
GLFW'ye bağlı yüzeyi — pencere oluşturma, monitör/video modu seçimi, fullscreen
geçişleri, framebuffer resize geri çağrıları, odak ve DPI davranışı — açıktır ve
kendi dilimlerinde ele alınır. Kullanıcı ayarlarından gelen GUI ölçeği seçimi ve
bu değerlerin GUI çizimine bağlanması P22'dedir.
