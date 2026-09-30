# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; uzun vadeli uyumluluk hedefi [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin). Referans Java Edition **26.2**, protokol **776**, Java **25** olarak sabitlendi. Güncel bağlantı baseline'ı resmi vanilla 26.2 server ile login, configuration, ilk chunk ve ilk oyuncu konumu düzeltmesini doğruladı; chunk wire gövdesi de yapısal olarak tüketiliyor. Mekanik uyumluluk ve oynanabilir istemci henüz yok.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

Referans Minecraft Java Edition 26.2, protokol 776 ve Java 25 olarak sabittir. Headless Java oracle ve Rust adapter dilimleri, her biri kendi birleşik contract/parity kaydında izlenen küçük adımlarla ilerler.

Kapsamın kesin listesi ve capture kimlikleri [davranış runner'ında](docs/behavior-runner.md), kaynak kararları [source review'da](docs/source-review.md), açık bağımlılıklar [roadmap](docs/roadmap.md) ve [kalan test planında](docs/remaining-test-plan.md) bulunur. Bu genel bakışta hareketli test toplamı tutulmaz; yeni dilim eski durum paragraflarını güncellemeden kendi kanıtını ekler.

2026-09-12 tarihli [yön düzeltmesi](docs/course-correction-2026-09-12.md) kanıt sınıflarını (`client-parity` / `server-conformance`) ayırır ve item component varyant serisini durdurur. P08 istemci dünya kopyası başladı; P06–P08 ve P12–P13 altındaki canlı sunucu kayıtları `server-conformance` sınıfındadır ve bölümü kapatmazlar.

Rust uygulama girişi henüz oynanabilir istemci değildir. İlk gerçek vanilla bağlantı probe'u ve Pumpkin karşılaştırması [birleşik oyun oturumu kaydında](docs/game-connection-conformance.md) tutulur. Pencere/input, dünya state'i, hareket, render, ses ve GUI için istemciye bağlı doğrulamalar sonraki roadmap bölümlerindedir.

## Yerel çalıştırma

Referans oyunu `.data/` altına indirmek, test sunucusunu ve istemciyi
başlatmak için `python3 tools/voxel.py` kullanılır (Linux, macOS, Windows);
ayrıntı [yerel kurulum belgesinde](docs/local-setup.md). İndirilenler yalnız
geliştirme makinesinde kalır, dağıtılan istemciye girmez.

## Belgeler

Tüm davranış sözleşmeleri, parity/conformance kayıtları ve çalışma belgeleri
[belge dizininde](docs/index.md) tutulur. Dizin, `docs/` içeriğinden
`tools/generate-doc-index.py` ile üretilir. Crate katmanları, kod düzeni ve
karmaşıklık sınırları [kodlama standartlarındadır](docs/coding-standards.md).

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
