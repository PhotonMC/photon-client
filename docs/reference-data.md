# Yerel referans verileri

Mevcut set: Java Edition **26.2**. Doğrulanan kimlikler, dosya sayıları ve çalıştırılabilirlik sonucu [referans envanterinde](reference-baseline.md).

`.data/` bütünüyle Git dışında tutulur. Boş dizinler Git ile taşınmaz; başka bir checkout'ta gerektiğinde yeniden oluşturulur. Kaynak veya assetleri bu alanın dışına kopyalama.

Önerilen yerleşim:

```text
.data/
  minecraft/
    source/          # Java kaynakları ve varsa derleme dosyaları
    assets/          # Orijinal dizin yapısı korunmuş assetler
    runtime/         # Varsa çalıştırılabilir Java client, jar ve bağımlılıklar
    mappings/        # Varsa sürümle eşleşen mappings
  captures/          # Yerel Java trace, görüntü, ses ve golden çıktıları
  runs/              # Karşılaştırma raporları, loglar ve fark çıktıları
  reference-info.md # Yerel sürüm, köken ve çalıştırma notları
```

Dosyalar mevcut yapılarıyla da eklenebilir; bunları bu düzene dönüştürmek ön koşul değildir. Asset index ve hash tabanlı object yapısı varsa ikisini de koru. Kaynak ve assetlerin tam sürümünü, biliniyorsa mappings bilgisini ve Java'yı nasıl çalıştırdığını `reference-info.md` içine yaz. Kimlik doğrulama token'ı veya hesap parolası ekleme.

İlk envanter çıkarıldı; referans seti değiştiğinde yeniden doğrulanır. Davranış referansı için tam Java sürümü, kaynak/mappings kimliği, asset seti kimliği, Java runtime, platform, oyun ayarları ve gerekli girdilerin hash'leri yerel manifestte sabitlenir. Pumpkin aday revizyonundaki 776 eşlemesi kaynak üzerinden doğrulandı; ilk offline bağlantı/world-delivery probe'u çalıştırıldı, mekanik eşitlik ayrıca test edilecek.

Kaynak kodu tek başına bütün görsel, ses ve zamanlama testlerini çalıştırmaya yetmeyebilir. Çalıştırılabilir Java referansı ve gerekli runtime/assetler mevcutsa ajan bunları kullanarak otomasyonu kurar. Eksik bağımlılıklar önce envanter ve yerel ortamdan belirlenir; yalnızca çözülemeyen eksikler kullanıcıya bildirilir. Hazır olmayan testler açıkça engelli olarak raporlanır.

Özgün test çalıştırıcıları ve sentetik senaryolar depoda bulunabilir. Orijinal referans çıktıları, ekran görüntüleri, PCM verileri, dünya dosyaları ve içerik barındıran paket kayıtları `.data/` altında kalır. Genel CI bunları yüklemez; yerel eşitlik testlerinin çalışmaması CI başarısıyla gizlenmez.
