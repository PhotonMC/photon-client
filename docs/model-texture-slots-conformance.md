# P16.c — Model texture slot çözümü — [client-parity] birleşik kayıt

**Tarih:** 2026-09-26 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik model zinciri pinned
JAR'daki `TextureSlots.Resolver`'a ve üretim kütüphanesindeki
`voxel_client::render::model` modülüne verilir, çözülen slot→sprite eşlemesi
birebir karşılaştırılır. GPU ve asset gerektirmez.

Render veri katmanının üçüncü dilimi; öncekiler
[atlas paketleyici](atlas-stitcher-conformance.md) ve
[PNG çözme](png-decode-conformance.md).

## Java kaynak incelemesi

Bir model, yüzlerinin dokusunu ya doğrudan adlandırır ya da başında `#` olan
bir slot adıyla başka bir slota yönlendirir. `ResolvedModel` parent zincirini
çocuktan köke doğru yürüyüp her seviyeyi `Resolver.addLast` ile ekler.

`resolve` seviyeleri **ters** sırayla, yani kökten başlayarak uygular. Böylece
çocuk ebeveyni ezer. Her slot için:

- Doğrudan değer, o slot için bekleyen yönlendirmeyi siler ve değeri yazar.
- Yönlendirme, o slot için çözülmüş değeri siler ve yönlendirmeyi yazar.

Bu iki silme, bir çocuğun devraldığı türü değiştirebilmesini sağlar. Ardından
yönlendirmeler sabit noktaya kadar çözülür: hedefi çözülmüş olan her yönlendirme
değerini alır, değişiklik kaldığı sürece tekrarlanır. Artakalan yönlendirmeler
**hata değildir**; yalnızca loglanır ve slot sonuçtan düşer.

`getMaterial` aramadan önce baştaki `#` işaretini atar, yani bir yüz slotunu
iki biçimde de adlandırabilir.

## Girdiler

`model/texture-slots` 16 parametrik satır taşır: boş zincir, tek değer,
`force_translucent` içeren nesne biçimi, çocuğun ebeveyni ezmesi, gerçek
`cube_all` düzeni (ebeveyn her yüzü `#all`'a yönlendirir, çocuk doldurur),
üç seviyeli kalıtım, yönlendirme zinciri, değerin yönlendirmeyi ve
yönlendirmenin değeri temizlediği durumlar, hedefsiz yönlendirme, kendine
yönlendirme, iki slotluk döngü, çözülebilir bir yönlendirmenin yanındaki
döngü, kalıtılan `force_translucent` ve karışık seviyeler.

## Sonuçlar

Java ve Rust her satırda aynı slot kümesini, aynı sprite'ları ve aynı
`force_translucent` bayraklarını verdi. Kaydın ayırt edici satırları:

| Girdi | Sonuç | Ne gösteriyor |
| --- | --- | --- |
| `cube-all` | altı slot da `stone` | Ebeveynin `#all` yönlendirmeleri çocuğun değerinden çözülüyor |
| `reference-clears-value` | `all=dirt` | Çocuğun yönlendirmesi kalıtılan değeri siliyor |
| `value-clears-reference` | `all=stone`, `side=dirt` | Çocuğun değeri kalıtılan yönlendirmeyi siliyor |
| `dangling-reference`, `self-reference`, `cycle` | boş | Çözülemeyen yönlendirme hata değil, düşürme |
| `cycle-plus-resolvable` | yalnızca `bottom` | Döngü yanındaki sağlam yönlendirme etkilenmiyor |
| `deep-override` | en alttaki çocuk kazanıyor | Ters uygulama sırası doğrulandı |
| her satır | `all` ve `#all` aynı sonucu veriyor | `getMaterial`'ın `#` atması |

Koşu: `python3 tools/behavior/run.py capture --family model` ardından
`compare --family model --adapter target/release/examples/behavior_adapter`
→ `parity_passed`.

## Kapsam sınırı: JSON ayrıştırma bu dilimde değil

Girdiler bu kayıtta slot adı ve ham değer olarak verilir; baştaki `#` kuralı ve
sprite kimliği her iki tarafta da sınanır, ancak `force_translucent` içeren
nesne biçimi yazılı JSON'dan değil tipli alandan gelir. Üretim kütüphanesinde
henüz JSON ayrıştırıcı yoktur ve bu kasıtlıdır: ayrıştırma kendi dilimini hak
eder. Model dosyalarının okunması, parent zincirinin dosyalardan kurulması ve
`Material.CODEC`'in tam JSON biçimi o dilimde sabitlenecektir.

## Açık kapsam

Model geometrisi (`elements`, yüzler, UV'ler, döndürme), `ambientOcclusion` ve
`guiLight` kalıtımı, item dönüşümleri, blockstate varyant seçimi ve quad
pişirme açıktır. Boş dizgi bir slot değeri olarak sınanmadı; referansta
`charAt(0)` çağrıldığı için o girdi Java tarafında istisna üretir.
