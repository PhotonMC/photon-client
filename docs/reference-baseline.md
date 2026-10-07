# Referans envanteri — Java Edition 26.2

İnceleme tarihi: 7 Eylül 2026. Kimlikler yerel `manifest.json`, `runtime/26.2.json`, client JAR'ı ve içindeki `version.json` ile karşılaştırıldı. Kullanıcının belirttiği Mojang dağıtımı kökeni kaydedildi; bu turda Mojang'dan yeniden indirme yapılmadı.

## Sabitlenen girdiler

| Girdi | Doğrulanan değer |
| --- | --- |
| Java Edition | 26.2, release |
| Protokol | 776 |
| Dünya veri sürümü | 4903 |
| Resource pack sürümü | 88.0 |
| Data pack sürümü | 107.1 |
| Java gereksinimi | 25, `java-runtime-epsilon` |
| Yerel çalıştırılan JDK | OpenJDK 25.0.4.1, Linux x86_64 |
| Client giriş sınıfı | `net.minecraft.client.main.Main` |
| Client JAR boyutu | 39.193.383 bayt |
| Client SHA-1 | `2dc72797acbc1b63fc16a11c4ac393605f453754` |
| Sürüm metadata SHA-1 | `3592ebc61c6b6c33bb8228fe5a9e90221df0be68` |
| Asset index | 32 |
| Asset index SHA-1 | `795a52d29f7f6b1e51d3a65e60ca46ad62aaddec` |
| Java kaynak ağacı SHA-256 | `d56cecfa2060e1229c1a5795e2514777b75249195241ec1f8d21af65f03336b3` |

Kaynak ağacı hash'i: göreli POSIX yollarına göre sıralanmış her `.java` dosyası için `UTF-8(yol) + NUL + SHA-256(içerik)` ham baytlarının birleştirilmesinin SHA-256'sı. Mutlak kullanıcı yolu hash'e dahil değildir.

## Bütünlük sonucu

- 10.952 `.class` girişi ve 7.055 `.java` dosyası bulundu. Kaynak dosya sayısı iç sınıflar nedeniyle class sayısıyla aynı değildir.
- 5.057 asset nesnesinin tamamı index'teki boyut ve SHA-1 değerleriyle eşleşti; eksik veya farklı nesne yok.
- Sürüm metadata'sındaki 131 bağımlılık artifact'inin tamamı yerel `runtime/libs/` içinde boyut ve SHA-1 açısından eşleşti. Bu, her platformdaki native yüklemenin çalıştığı anlamına gelmez.
- JAR'ın class ve `META-INF/` dışındaki 19.497 dosyası `assets/jar/` ile içerik hash'i açısından eşleşti.
- Client, sürüm JSON'u ve asset index hash'leri manifest/metadata ile eşleşti.
- Kaynaklarda Vineflower'ın decompile edilemeyen metot işareti bulunmadı. Bu tarama decompiler'ın bütün ifadeleri semantik olarak doğru yeniden kurduğunu kanıtlamaz.

Ham denetim raporu: `.data/runs/reference-audit.json`. Orijinal veri, dosya bazlı çıktılar ve gelecekteki referans yakalamaları yerel kalır.

Kaynaklar kullanıcı notuna göre Vineflower 1.12.0 ile bu JAR'dan çıkarılmıştır. Bu setin sınıf/metot adları okunabilir; ayrı mappings girdisi bulunmuyor. Decompile edilmiş kaynak, inceleme aracıdır. Anlaşmazlıkta çalıştırılan orijinal bytecode ve kontrollü gözlem belirleyicidir; bütün decompile ağacını yeniden derlemek ön koşul değildir.

## Referans çalıştırılabilirliği

Orijinal client JAR'ı ve mevcut bağımlılıklar kullanılarak grafik arayüzsüz küçük bir yerel yoklama derlendi ve iki kez çalıştırıldı:

| Orijinal sınıf | Yoklama | Sonuç |
| --- | --- | --- |
| `DeltaTracker.Timer` | Kontrollü başlangıç zamanı ve zaman ilerletme | Çağrılabildi; iki çalıştırma aynı çıktı |
| `Vec2` | Diyagonal vektör normalizasyonu, ham float bitleri | Çağrılabildi; iki çalıştırma aynı çıktı |
| `VarInt` | Negatif sayının Netty buffer'a yazılıp okunması | Çağrılabildi; iki çalıştırma aynı çıktı |

Yoklama kodu ve sonuçları `.data/runs/reference-probe/` içinde. Bu ilk yoklama Rust eşitlik kanıtı değildir. Sonraki çalışmada [kalıcı çalıştırıcı ve 346 senaryoluk corpus](behavior-runner.md) hazırlandı; aşağıdaki yoklama bilgileri ilk incelemenin kaydıdır. İlk testler için Java sınıflarını doğrudan çağırmanın uygulanabilir olduğunu gösterir. İlk envanter sırasında tam istemci, GPU, ses çıkışı ve gerçek oyuncu oturumu denenmedi; güncel vanilla bağlantı/world-delivery sonucu ve önceki Pumpkin koşusu [birleşik oyun oturumu kaydında](records/game-connection-conformance.md) tutulur. Kullanıcının Prism instance'ına veya ayarlarına dokunulmadı.

## Asset yükleme açısından sonuç

`assets/objects` ve index çoğunlukla ses/dil kaynaklarını, `assets/jar/` ise texture, model, blockstate, shader, font ve veri kaynaklarını tamamlıyor. `assets/named/` gezinme amaçlı symlink görünümü; yeni bir asset seti değildir.

Java'nın `ClientPackSource.createVanillaPackSource` yolu JAR kaynaklarını ve haricî asset kökünü birlikte kullanıyor. Dolayısıyla ilerideki yerel klasör seçimi, bu iki kaynağı ve sürümünü çözebilmeli. Yalnızca `sounds/` veya `objects/` klasörünü seçmek tam eşitlik için yeterli değil. Paket önceliği, metadata, yerleşik pack'ler ve eksik kaynak davranışı P04'ün test konusu. Orijinal shaderlar da asset olarak yerelde kalacak; Rust kaynaklarına veya dağıtıma kopyalanmayacak.

## Pumpkin incelemesi

İncelenen revizyon: `786e2fbf782f92113543fcb46eaf95ba53835d41` (commit tarihi 6 Eylül 2026). Bu, ilk entegrasyon denemeleri için kayıtlı adaydır; henüz başarıyla çalıştırılmış sunucu baseline'ı değildir.

Bu revizyonun `JavaMinecraftVersion` tanımı `V_26_2` ile `776` arasında iki yönlü eşleme içeriyor. Codegen tarafında da 26.2 varyantı var. Bu kanıt yalnızca sürümün kodda tanındığını gösterir. [Sürüm eşlemesi](https://github.com/Pumpkin-Mc/Pumpkin/blob/786e2fbf782f92113543fcb46eaf95ba53835d41/crates/pumpkin-util/src/version.rs#L141), [codegen varyantı](https://github.com/Pumpkin-Mc/Pumpkin/blob/786e2fbf782f92113543fcb46eaf95ba53835d41/tools/pumpkin-codegen/src/version.rs#L196).

Sunucunun kendi README'si geliştirme aşamasında olduğunu ve tamamlanmamış mekanikler bulunduğunu belirtiyor. Protokol numarasının tanınması bütün paketlerin, registry verilerinin veya oyun mekaniklerinin eşitliğine kanıt değildir. [Sabit revizyondaki README](https://github.com/Pumpkin-Mc/Pumpkin/blob/786e2fbf782f92113543fcb46eaf95ba53835d41/README.md).

İnceleme kopyaları `.data/runs/pumpkin-review/` içinde. Bu envanter incelemesinde Pumpkin sunucu kodu projeye dahil edilmedi. Güncel test baseline'ı Mojang vanilla 26.2 server ile ve Pumpkin koşusu [birleşik oyun oturumu kaydında](records/game-connection-conformance.md) ayrı alt bölümlerde tutulur. Her iki probe da [P06–P08 bağlantı sözleşmesinin](records/game-connection-conformance.md) dar kapsamındadır; P06–P08'in geri kalan ve P26 oyun senaryolarıyla uyumluluk ayrıca ölçülecek.

## Henüz doğrulanmayanlar

Tam grafik/ses referansının otomasyonu, Linux'taki renderer ve ses cihazı ayarları, diğer işletim sistemleri, gerçek bağlantı, hesap gerektiren servisler ve tek oyunculu akışın Pumpkin ile eşitliği açık. Bunlar ilk grafik arayüzsüz testleri engellemiyor; tamamlanmış istemci eşitliği iddiasını engelliyor.
