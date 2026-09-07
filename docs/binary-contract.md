# P03.b — UUID ve BlockPos referans sözleşmesi

Bu dilimin **43 Java davranış testi hazırdır; Rust implementasyonu henüz yoktur**. Önceki 487 math/timer/wire/utf8 senaryosu değişmedi. Long array, BitSet ve diğer P03 biçimleri bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `network/FriendlyByteBuf.java` içindeki `readUUID`/`writeUUID` ve `readBlockPos`/`writeBlockPos`, ayrıca `core/BlockPos.java` içindeki `asLong`, `of` ve koordinat çıkarma incelendi. Adaptör doğrudan orijinal sınıfları çağırır; üst katman paket mantığı veya gerçek ağ bağlantısı bu testte yapılmaz.

Netty buffer/common yanında `core/BlockPos` sınıfı için math ailesinin bağımlılıkları ve `commons-lang3` gerekir. Yerel JAR'ın SHA-1 değeri `tests/behavior/reference.json` içine sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- UUID iki big-endian i64 olarak taşınır: önce most, sonra least. Herhangi bir 16 bayt geçerlidir; uzunluk öneki ve sürüm/varyant denetimi yoktur.
- BlockPos tek big-endian i64 olarak taşınır. X/Z 26 bit, Y 12 bit işaretli alanlara maskelenir; aralık dışı i32 girdiler sarar. Açılım aritmetik kaydırma ile işaret genişletir.
- Okuma her `readLong` için atomiktir: yetersiz veri tüketimi ilerletmez. UUID iki ardışık okumadır; ikinci yarı eksikse ilk 8 bayt tüketilmiş olur. BlockPos tek okumadır; eksik veride tüketim olmaz.
- Yazma bu dilimde her zaman başarılıdır; uzunluk sınırı ve hata kategorisi yoktur. Mevcut yazma sınırı hatalarında hedef buffer'a yeni veri eklenmez kuralı burada tetiklenmez.
- Okuma hatası yalnızca `buffer_underflow` kategorisidir. Java exception sınıfı yalnızca `diagnostics` içindedir; mesaj/stack eşitlik ölçütü değildir.

İlk corpus `prefix_hex` olarak `a55a` kullanır; heap buffer ve sıfırdan farklı reader index ile çalışır. Direct/composite buffer varyantları ve çoklu değer akışları kapsam dışıdır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| UUID yazma | 8 | Sıfır/tüm-bir, örnek, alterne bitler, sürüm benzeri, tek bit ve `deadbeef` desenleri |
| UUID okuma | 9 | 0/1/7/8/9/15 baytlık kesikler, tam 16 bayt ve tüm-bir yük; kalan veri `55aa` ile sınanır |
| BlockPos yazma | 12 | Köken/birim/negatif, yatay/Y sınırları ve sarma komşuları, dünya sınırı, i32 uçları ve karışık koordinat |
| BlockPos okuma | 14 | Bağımsız 8 baytlık yükler ve 0–7 baytlık kesikler; kalan veri `55aa` ile sınanır |

`family: "binary"`, `operation: "read" | "write"`, `codec: "uuid" | "blockpos"` ortak girdilerdir. `prefix_hex` küçük harfli bayt hex stringdir. UUID yazmada `most_hex`/`least_hex` 16 haneli küçük harfli hex; BlockPos yazmada `x`/`y`/`z` işaretli ondalık stringdir. Okumada `hex` başlık/gövdeyi taşır.

Okuma adaptörü buffer başına `prefix_hex` koyup reader index'i bu alanın sonuna taşır. Başarılı UUID sonucu `most_hex`, `least_hex`, `outcome`; başarılı BlockPos sonucu `x`, `y`, `z`, `outcome` içerir. Her iki durumda `consumed` başlangıç reader index'ine göre tüketimi, `remaining_hex` kalan veriyi gösterir. Hata sonucu `outcome` ve `error` içerir.

Yazma adaptörü mevcut `prefix_hex` üzerine ekler. Her sonuç `outcome`, eklenen bayt sayısı `written` ve prefix dahil bütün `hex` çıktısını içerir. Bu dilimde yazma hatası beklenmez; bağımsız okuma senaryoları ayrıdır.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family binary
python3 tools/behavior/run.py verify --family binary
```

43 senaryo iki ayrı JVM'de aynı sonucu verdi; kayıtlı referans iki yeni JVM ile yeniden doğrulandı. Binary referans kimliği: `f320872a5c6e2a905ccea0ceb896ff62c7ce4ffeac34433b21563287d815d749`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Dispatcher'a binary eklenmesi kaynak kimliğini değiştirdiği için önceki dört aile de yeni kimlikle açıkça yakalandı; eski 487 girdinin ve sonuçlarının önceki kayıtla aynı olduğu ayrıca kontrol edildi. Yeni 487 kimliği: `7a44e1f158276bac917dc6eef25d333192348f93ebbd46b73d26b2c94842a6c3`. Yeni aileyle toplam 530 Java senaryosu yakalandı ve iki yeni JVM çalıştırmasıyla kayıtla eşitliği doğrulandı. Birleşik referans kimliği: `88117b64f9b5f22bcca4fa95b542409ee781a26606da3861746e9673fd4fac3a`. Eski kayıtlar değiştirilmedi.

Mevcut Rust kodunu çalıştırmak için desteklenen aileleri açıkça seç:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --family timer --family wire --family utf8 --adapter target/debug/examples/behavior_adapter
```

Varsayılan bütün-aile karşılaştırması binary henüz uygulanmadığından başarısız olur; yeni testler sessizce atlanmaz.

Sonraki adım bu sabit referansa karşı binary Rust davranış dilimidir. Beklenen çıktılar Rust'a göre değiştirilmeyecek.
