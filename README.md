# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin) kullanılacak. Referans Java Edition **26.2**, protokol **776**, Java **25** olarak sabitlendi. İncelenen Pumpkin adayında 776 sürüm eşlemesi mevcut; gerçek bağlantı ve mekanik uyumluluğu henüz doğrulanmadı.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

Kaynaklar sağlandı; bütünlük envanteri ve ilk kaynak incelemesi tamamlandı. Bölüm bazlı roadmap ve ilk davranış testi sözleşmeleri hazır. P00 yerel test çalıştırıcısı ve 910 Java referans senaryosu mevcut. Random/AABB, AABB contract, AABB inflate, AABB intersect, AABB predicate overload'ları, AABB collidedAlongVector, AABB minmax, AABB move, AABB move overload'ları, AABB sizes, AABB hasNaN, AABB positions, AABB axis min/max, AABB distanceToSqr, AABB ofSize, AABB endpoint setter'ları, AABB clip, VoxelShape temel/OR, BooleanOp join ve tekli/çoklu shape collision, nextGaussian, fork/positional, setSeed, random yardımcı overload'ları, NBT ağaç/liste/dizi ve BlockPos/section-relative dilimleri dahil Rust uygulanan kapsam debug/release profillerinde 910 senaryonun tamamında Java ile eşleşiyor. Oynanabilir istemci henüz yok. Boş `main`, yalnızca iskelettir.

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
- [AABB contract sözleşmesi ve referans testleri](docs/aabb-contract.md)
- [AABB contract doğrulaması](docs/aabb-parity.md)
- [AABB inflate sözleşmesi ve referans testleri](docs/aabb-inflate-contract.md)
- [AABB inflate doğrulaması](docs/aabb-inflate-parity.md)
- [AABB intersect sözleşmesi ve referans testleri](docs/aabb-intersect-contract.md)
- [AABB intersect doğrulaması](docs/aabb-intersect-parity.md)
- [AABB predicate overload sözleşmesi ve referans testleri](docs/aabb-predicate-overloads-contract.md)
- [AABB predicate overload doğrulaması](docs/aabb-predicate-overloads-parity.md)
- [AABB collidedAlongVector sözleşmesi ve referans testleri](docs/aabb-collided-along-vector-contract.md)
- [AABB collidedAlongVector doğrulaması](docs/aabb-collided-along-vector-parity.md)
- [AABB minmax sözleşmesi ve referans testleri](docs/aabb-minmax-contract.md)
- [AABB minmax doğrulaması](docs/aabb-minmax-parity.md)
- [AABB move sözleşmesi ve referans testleri](docs/aabb-move-contract.md)
- [AABB move doğrulaması](docs/aabb-move-parity.md)
- [AABB move overload sözleşmesi ve referans testleri](docs/aabb-move-overloads-contract.md)
- [AABB move overload doğrulaması](docs/aabb-move-overloads-parity.md)
- [AABB sizes sözleşmesi ve referans testleri](docs/aabb-sizes-contract.md)
- [AABB sizes doğrulaması](docs/aabb-sizes-parity.md)
- [AABB hasNaN sözleşmesi ve referans testleri](docs/aabb-has-nan-contract.md)
- [AABB hasNaN doğrulaması](docs/aabb-has-nan-parity.md)
- [AABB positions sözleşmesi ve referans testleri](docs/aabb-positions-contract.md)
- [AABB positions doğrulaması](docs/aabb-positions-parity.md)
- [AABB axis min/max sözleşmesi ve referans testleri](docs/aabb-axis-limits-contract.md)
- [AABB axis min/max doğrulaması](docs/aabb-axis-limits-parity.md)
- [AABB distanceToSqr sözleşmesi ve referans testleri](docs/aabb-distance-contract.md)
- [AABB distanceToSqr doğrulaması](docs/aabb-distance-parity.md)
- [AABB ofSize sözleşmesi ve referans testleri](docs/aabb-of-size-contract.md)
- [AABB ofSize doğrulaması](docs/aabb-of-size-parity.md)
- [AABB endpoint setter sözleşmesi ve referans testleri](docs/aabb-set-endpoint-contract.md)
- [AABB endpoint setter doğrulaması](docs/aabb-set-endpoint-parity.md)
- [AABB clip sözleşmesi ve referans testleri](docs/aabb-clip-contract.md)
- [AABB clip doğrulaması](docs/aabb-clip-parity.md)
- [VoxelShape temel/OR doğrulaması](docs/shape-parity.md)
- [VoxelShape BooleanOp doğrulaması](docs/shape-boolean-parity.md)
- [VoxelShape collision doğrulaması](docs/shape-collision-parity.md)
- [VoxelShape çoklu collision doğrulaması](docs/shape-collision-many-parity.md)
- [`nextGaussian` sözleşmesi ve referans testleri](docs/random-gaussian-contract.md)
- [`nextGaussian` doğrulaması](docs/random-gaussian-parity.md)
- [`fork` ve positional random sözleşmesi](docs/random-fork-contract.md)
- [`fork` ve positional random doğrulaması](docs/random-fork-parity.md)
- [`setSeed` sözleşmesi](docs/random-set-seed-contract.md)
- [`setSeed` doğrulaması](docs/random-set-seed-parity.md)
- [Random yardımcı overload sözleşmesi](docs/random-overloads-contract.md)
- [Random yardımcı overload doğrulaması](docs/random-overloads-parity.md)
- [Kalan bölümler test planı](docs/remaining-test-plan.md)
- [NBT davranış sözleşmesi ve referans testleri](docs/nbt-contract.md)
- [NBT doğrulaması](docs/nbt-parity.md)
- [UTF-8 doğrulaması](docs/utf8-parity.md)
- [UTF-8 davranış sözleşmesi ve referans testleri](docs/utf8-contract.md)
- [UUID/BlockPos doğrulaması](docs/binary-parity.md)
- [UUID/BlockPos davranış sözleşmesi ve referans testleri](docs/binary-contract.md)

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
