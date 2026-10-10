# Çerez, özel yük ve sorgu paketleri — [server-conformance değil, birim ve sahte sunucu testi]

Kapsam: giriş, yapılandırma ve oyun evrelerinde `CookieRequest` (yanıt: anahtarın altındaki bayt dizisi ya
da yok), `StoreCookie` (en çok 5120 bayt), istemciye gelen `CustomPayload` (`minecraft:brand` saklanır,
öteki yükler işlenmez), giriş evresinde `CustomQuery` (yanıtsız cevap, `handleCustomQuery`) ve
`CustomReportDetails` (en çok 32 çift; anahtar 128, değer 4096 karakter). Çerezler yeniden yapılandırmada
(T101) korunur.

Kanıt: `cookie` modülü birim testleri (kodlama, sınırlar, marka, yanıtsız sorgu) ve oyun evresinde sahte
sunucu testi (çerezi sakla, iste, yanıtı doğrula). Vanilla sunucuyla bir oturum bozulmadan açıldı; çerez
isteyen bir sunucu elde yok. Java oracle'ı yok; `client-parity` değildir.

## Sapmalar (açık)

- Sunucu bağlantıları (`ServerLinks`) çözülmez: bileşen çözümü gerekir, T106'dadır.
- Özel yük içeriği (marka dışında) ve rapor ayrıntıları yalnız saklanır; hiçbir ekran göstermez.
- Çerezler yalnız bellektedir; transferde taşınması T102'dedir.
