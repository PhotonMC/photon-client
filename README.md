# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin) kullanılacak. Referans Java Edition **26.2**, protokol **776**, Java **25** olarak sabitlendi. İncelenen Pumpkin adayında 776 sürüm eşlemesi mevcut; gerçek bağlantı ve mekanik uyumluluğu henüz doğrulanmadı.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

Kaynaklar sağlandı; bütünlük envanteri ve ilk kaynak incelemesi tamamlandı. Bölüm bazlı roadmap ve ilk davranış testi sözleşmeleri hazır. P00 yerel test çalıştırıcısı ve 783 Java referans senaryosu mevcut. Random/AABB, AABB clip, nextGaussian, NBT ağaç/liste/dizi ve BlockPos/section-relative dilimleri dahil Rust uygulanan kapsam debug/release profillerinde 783 senaryonun tamamında Java ile eşleşiyor. Oynanabilir istemci henüz yok. Boş `main`, yalnızca iskelettir.

Mevcut referanslar [yerel veri rehberindeki](docs/reference-data.md) `.data/` alanında kalır. Test yol haritası kalan P01 yardımcıları ve bağlantı önkoşullarıyla sürüyor. Her Rust diliminden önce ilgili Java referansı ve testler hazırlanacak. Grafik, ses, pencere ve ağ teknolojileri sonraya bırakıldı. Test otomasyonu Python standart kütüphanesi ve özgün Java adaptörlerini kullanıyor; üretim kütüphanesine ek Rust bağımlılığı eklenmedi. Rust JSONL test adaptörü `serde_json` geliştirme bağımlılığını kullanıyor.

## Belgeler

- [Tüm ajanlar için çalışma kuralları](AGENTS.md)
- [Yerel kaynak ve asset yerleşimi](docs/reference-data.md)
- [Davranış doğrulama yaklaşımı](docs/behavior-testing.md)
- [Doğrulanan referans envanteri ve Pumpkin adayı](docs/reference-baseline.md)
- [Kaynak incelemesi ve istemci sınırları](docs/source-review.md)
- [Bölüm bazlı roadmap](docs/roadmap.md)
- [İlk davranış testi sözleşmeleri](docs/first-behavior-tests.md)
- [Yerel test komutları ve sonuçların anlamı](docs/behavior-runner.md)
- [VarInt/VarLong doğrulaması](docs/wire-parity.md)
- [Zamanlayıcı doğrulaması](docs/timer-parity.md)
- [Vec2 ve sayısal yardımcıların doğrulaması](docs/math-parity.md)
- [Vec3 davranış sözleşmesi ve referans testleri](docs/vec3-contract.md)
- [Direction davranış sözleşmesi ve referans testleri](docs/direction-contract.md)
- [Koordinat dönüşümleri sözleşmesi ve referans testleri](docs/coords-contract.md)
- [Random/AABB sözleşmesi ve referans testleri](docs/random-aabb-contract.md)
- [BlockPos/section-relative sözleşmesi ve referans testleri](docs/blockpos-contract.md)
- [BlockPos/section-relative doğrulaması](docs/blockpos-parity.md)
- [AABB clip sözleşmesi ve referans testleri](docs/aabb-clip-contract.md)
- [AABB clip doğrulaması](docs/aabb-clip-parity.md)
- [`nextGaussian` sözleşmesi ve referans testleri](docs/random-gaussian-contract.md)
- [`nextGaussian` doğrulaması](docs/random-gaussian-parity.md)
- [Kalan bölümler test planı](docs/remaining-test-plan.md)
- [NBT davranış sözleşmesi ve referans testleri](docs/nbt-contract.md)
- [NBT doğrulaması](docs/nbt-parity.md)
- [UTF-8 doğrulaması](docs/utf8-parity.md)
- [UTF-8 davranış sözleşmesi ve referans testleri](docs/utf8-contract.md)
- [UUID/BlockPos doğrulaması](docs/binary-parity.md)
- [UUID/BlockPos davranış sözleşmesi ve referans testleri](docs/binary-contract.md)

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
