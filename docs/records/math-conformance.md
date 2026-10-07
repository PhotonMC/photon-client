# P01 — Math ailesi doğrulaması — [client-parity] kapsam kaydı

Bu belge, Java Edition 26.2 / protokol 776 / Java 25 referansından alınan sayısal ve geometrik davranışların Rust karşılaştırma yöntemini açıklar. Uygulanan her küçük dilimin kesin sınırı kendi birleşik contract/parity kaydındadır.

## Referans ve implementasyon

Java oracle doğrudan sabitlenmiş client runtime sınıflarını çağırır. Rust JSONL adapter'ı üretim modüllerini çağırır; beklenen değerleri yeniden hesaplayan ayrı bir test algoritması kullanmaz. Kaynak kararları [source review](../source-review.md), senaryo üretimi tools/behavior/cases.py ve genel JSONL kuralları [behavior runner](../behavior-runner.md) içinde tutulur.

Math modülü Vec2/Vec3, Direction, koordinat ve shape yardımcılarını; Mth sayısal yollarını; random modülü ise legacy random akışlarını taşır. Birbirine bağlı yardımcılar ortak üretim fonksiyonlarını kullanır. Örneğin [Mth getSeed](mth-get-seed-conformance.md) doğrudan math yardımcısını doğrular ve positional factory aynı yardımcıyı çağırır.

Her dilim Java kaynak sırasını, taşma genişliğini, float/double ham bitlerini, hata yolunu ve koleksiyon sırasını kendi sözleşmesinde belirtir. Eski dilim kanıtları yeni bir operation eklendiğinde bu genel özete kopyalanmaz.

## Doğrulama

Java referansı iki ayrı JVM çalıştırmasıyla kararlılık açısından kontrol edilir. capture mevcut kimliği değiştirmez; verify kayıtlı capture'ı yeniden Java ile kontrol eder; compare aynı capture'ı seçilen Rust adapter'ına karşı kesin olarak karşılaştırır.

~~~bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
~~~

Exact capture kimlikleri ve koşu raporları .data/ altında kalır. Debug/release sonuçları ilgili koşu raporlarında bulunur; bu belge hareketli corpus toplamlarını taşımaz.

## Kapsam sınırı

Bu kanıt yalnızca capture'a giren headless yardımcı ve veri davranışları içindir. Oynanabilir istemci, gerçek pencere/input, Pumpkin oturumu, GPU frame yakalama, ses capture'ı ve tam user-facing akışlar için henüz parity sonucu yoktur. Yeni bir math operation'ı için source review, kendi Java oracle senaryoları ve ayrı birleşik contract/parity kaydı zorunludur.
