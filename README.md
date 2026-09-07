# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin) kullanılacak. Referans Java Edition **26.2**, protokol **776**, Java **25** olarak sabitlendi. İncelenen Pumpkin adayında 776 sürüm eşlemesi mevcut; gerçek bağlantı ve mekanik uyumluluğu henüz doğrulanmadı.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

Kaynaklar sağlandı; bütünlük envanteri ve ilk kaynak incelemesi tamamlandı. Bölüm bazlı roadmap ve ilk davranış testi sözleşmeleri hazır. Henüz oyun implementasyonu veya kalıcı davranış test paketi yok. Boş `main`, yalnızca iskelettir.

Mevcut referanslar [yerel veri rehberindeki](docs/reference-data.md) `.data/` alanında kalır. Sıradaki iş P00 Java referans çalıştırıcısı; ardından sayısal işlemler, zamanlayıcı ve wire kodlamalarının davranış testleridir. Her Rust diliminden önce ilgili Java referansı ve testler hazırlanacak. Grafik, ses, pencere, ağ ve test teknolojileri sonraya bırakıldı; ek bağımlılık seçilmedi.

## Belgeler

- [Tüm ajanlar için çalışma kuralları](AGENTS.md)
- [Yerel kaynak ve asset yerleşimi](docs/reference-data.md)
- [Davranış doğrulama yaklaşımı](docs/behavior-testing.md)
- [Doğrulanan referans envanteri ve Pumpkin adayı](docs/reference-baseline.md)
- [Kaynak incelemesi ve istemci sınırları](docs/source-review.md)
- [Bölüm bazlı roadmap](docs/roadmap.md)
- [İlk davranış testi sözleşmeleri](docs/first-behavior-tests.md)

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
