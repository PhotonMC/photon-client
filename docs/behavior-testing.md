# Davranış doğrulama sözleşmesi

Bu belge yaklaşımı tanımlar. 26.2 kaynakları incelendi; [ilk test sözleşmeleri](first-behavior-tests.md) hazırlandı. [Kalıcı yerel çalıştırıcı](behavior-runner.md) ve altı ailede 815 Java referans senaryosu mevcut. Rust wire, zamanlayıcı, matematik, UTF-8, binary ve NBT adaptörleri uygulanan kapsamda toplam 815 senaryoda Java ile eşleşti; [wire kapsamı](wire-parity.md), [zamanlayıcı kapsamı](timer-parity.md), [sayısal kapsam](math-parity.md), [random/AABB kapsamı](random-aabb-parity.md), [BlockPos/section-relative kapsamı](blockpos-parity.md), [AABB clip kapsamı](aabb-clip-parity.md), [VoxelShape kapsamı](shape-parity.md), [VoxelShape BooleanOp kapsamı](shape-boolean-parity.md), [VoxelShape collision kapsamı](shape-collision-parity.md), [nextGaussian kapsamı](random-gaussian-parity.md), [fork/positional kapsamı](random-fork-parity.md), [setSeed kapsamı](random-set-seed-parity.md), [random overload kapsamı](random-overloads-parity.md), [UTF-8 kapsamı](utf8-parity.md), [binary kapsamı](binary-parity.md), [NBT kapsamı](nbt-parity.md). Yalnızca Java referansını doğrulamak Rust eşitlik testi değildir.

## Referans ve karşılaştırma

Referans, sabitlenecek Minecraft Java Edition sürümüdür. Her davranış dilimi için kaynaklar incelenir; aynı kontrollü senaryo Java ve Rust üzerinde çalıştırılarak gözlenebilir sonuçlar karşılaştırılır. Testin beklenen sonucunun kökeni Java referansı olmalıdır. Sadece kaynak yorumundan çıkarılan beklentiler, çalışan Java ile doğrulanana kadar bu eksikle işaretlenir.

Her test kaydı davranış kimliğini, kaynak konumunu (kod alıntısı olmadan), sürüm kimliklerini, ön koşulları, girdileri, gözlenen çıktıları, karşılaştırma kuralını ve çalıştırma yöntemini içerir. Referans yakalama, senaryo oynatma, karşılaştırma ve fark raporlama otomatik olmalıdır. Gerekli yerel kaynaklar sağlandıktan sonra kullanıcıdan elle test yürütmesi beklenmez.

Java doğrulanmadan Rust'a uygun golden oluşturulmaz. Referans yenileme açık bir işlem olmalı; test çalıştırması sırasında mevcut beklentiler sessizce değiştirilmemelidir. Aynı Java senaryosu tekrarlanarak referansın kararlılığı da değerlendirilir.

## Kontrol edilecek koşullar

İlgili olduğu ölçüde seed, dünya/oyuncu durumu, input zamanları, tick, frame süresi, ağ olaylarının sırası, dil, çözünürlük, GUI ölçeği, FOV, ses ayarları ve asset kimliği sabitlenir. Saat ve rastlantısallık kontrolü oyun semantiğini değiştirmemelidir. Java'nın taşma, kayan nokta, yuvarlama, işlem sırası ve olay sırası davranışları Rust varsayımlarıyla değiştirilmez.

Durum geçişleri, sayılar, olaylar ve protokol alanlarında sözleşmenin gerektirdiği kesin eşitlik kullanılır. Görüntü/ses karşılaştırmaları sabit senaryolar, kamera, animasyon evresi ve zamanlama üzerinden yapılır. Platform/GPU kaynaklı değişkenlik önce aynı Java referansının tekrarlı ölçümleriyle ortaya konur. Evrensel epsilon, piksel toleransı veya "yeterince benzer" ölçütü varsayılmaz; zorunlu bir tolerans varsa gerekçesi ve sınırı davranış sözleşmesinde açıkça belirtilir. Oyun farkını toleransla gizlemek kabul edilmez.

Otomatik görüntü ve ses ölçümleri, insan algısında fark olmadığına tek başına mutlak kanıt değildir. Doğrulama iddiası yalnızca ölçülmüş senaryo ve ortamlarla sınırlandırılır; hedef yine birebir eşitliktir.

## Dilim başına sıra ve tamamlanma

1. Java kaynaklarından davranışı ve bağımlılıklarını çıkar; kontrollü senaryoyu tanımla.
2. Yerel Java referansında senaryoyu çalıştır, çıktının kökenini ve kararlılığını kaydet.
3. Rust implementasyonundan önce karşılaştırma testini hazırla. Testin eksik/yanlış çıktıyı yakaladığını doğrula; henüz olmayan Rust adaptörünü başarılı sayma.
4. En küçük gerekli Rust dilimini uygula ve aynı girdilerle karşılaştır.
5. Fark varsa nedenini çöz; beklentiyi Rust'a uydurma. İlgili önceki davranış testlerini de çalıştır.
6. Doğrulanan kapsamı ve açık boşlukları kaydet; tamamlanmış dilimi ayrı commit yap.

İstemciye verilen aynı sunucu olaylarıyla davranış testi ve gerçek Pumpkin bağlantısıyla entegrasyon doğrulaması ayrı değerlendirilir. Testte olay beslemek üretim sunucusu geliştirmek anlamına gelmez. Pumpkin'in eksik veya farklı bir davranışı Java istemci eşitliğinin kanıtı değildir; farkın hangi taraftan geldiği kaydedilir.

Her senaryo geçti, kaldı veya engelli olarak raporlanır. Eksik Java, asset, ekran/ses ortamı veya Pumpkin desteği nedeniyle çalışmayan test, eşitlik onayı vermez. Başarısızlık raporları girdileri, ilk ayrışmayı ve yeniden çalıştırma bilgisini yerelde saklar.

Davranış kapsamı gerektiği kadar ayrıntılı olabilir. Kod kalitesi için tekrarlı mikro testler ve gereksiz kapsam yüzdesi hedefleri eklenmez. Test altyapısı ihtiyaç ortaya çıktıkça küçük adımlarla büyür.
