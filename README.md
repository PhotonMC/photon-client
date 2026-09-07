# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin) kullanılacak. Hedef Java sürümü ve Pumpkin revizyonu henüz seçilmedi; uyumluluk doğrulanmış değildir.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

Kaynaklar bekleniyor. Yalnızca Cargo iskeleti ve çalışma belgeleri mevcut. Henüz oyun implementasyonu, test altyapısı, davranış testi veya kaynağa dayalı ayrıntılı roadmap yok. Boş `main`, yalnızca iskelettir.

Referansları [yerel veri rehberine](docs/reference-data.md) göre `.data/` içine ekleyin. Ardından kaynak incelemesiyle bölümler ve bağımlılıklar belirlenecek, roadmap doldurulacak, her bölümün davranış testleri Rust implementasyonundan önce hazırlanacak. Grafik, ses, pencere, ağ ve test teknolojileri sonraya bırakıldı; ek bağımlılık seçilmedi.

## Belgeler

- [Tüm ajanlar için çalışma kuralları](AGENTS.md)
- [Yerel kaynak ve asset yerleşimi](docs/reference-data.md)
- [Davranış doğrulama yaklaşımı](docs/behavior-testing.md)
- [Roadmap: kaynak bekleniyor](docs/roadmap.md)

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
