# Roadmap — kaynaklar bekleniyor

Durum: hazırlık iskeleti oluşturuldu. Hedef Java sürümü, kaynak envanteri, Pumpkin revizyonu, bölüm sınırları ve teknoloji seçimleri henüz belirlenmedi.

Bu aşamada motor/mimari tasarımı, tahminî modül listesi veya implementasyon sırası oluşturulmaz. Kullanıcı `.data/` içine referansları ekledikten sonra kaynaklar incelenerek bu belge doldurulur.

İlerleme sırası:

1. Yerel kaynak, asset ve çalıştırılabilir referans envanterini çıkar; hedef sürümü ve kökeni sabitle.
2. Kaynakları bölüm bölüm incele; istemci sınırlarını, bağımlılıkları ve gözlenebilir Java davranışlarını belirle. Pumpkin protokol uyumluluğunu doğrula.
3. İncelemeye dayanarak küçük, bağımlılık sırası belli implementasyon dilimleri oluştur.
4. Her dilim için önce otomatik Java referansı ve davranış testlerini hazırla; sonra Rust implementasyonunu karşılaştırarak geliştir.
5. Eşitliği doğrulanan dilimi commit et. Davranışı koruma kanıtı olmadan optimizasyona geçme.

Kaynak incelemesinden sonra her dilimin kaydı şu alanları içerecek:

| Alan | İçerik |
| --- | --- |
| Kimlik ve kapsam | Tek incelenebilir davranış dilimi, kapsam dışı kısımlar |
| Java kanıtı | Tam sürüm, ilgili dosya/sembol konumları ve kendi ifadelerimizle davranış |
| Bağımlılıklar | Önce tamamlanması gereken dilimler ve gereken yerel girdiler |
| Davranış testleri | Kontrollü senaryolar, Java referans yakalama yöntemi, karşılaştırma sözleşmesi |
| Rust çalışması | Testler hazır olduktan sonra yapılacak en küçük implementasyon |
| Kabul ölçütü | Hangi senaryo ve ortamlarda hangi sonuçların eşit olması gerektiği |
| Durum ve kanıt | İncelenmedi / test hazırlanıyor / implementasyon / doğrulandı / engelli; sonuç ve commit |

Bu şablon henüz herhangi bir bölümün incelendiği, test edildiği veya tamamlandığı anlamına gelmez.
