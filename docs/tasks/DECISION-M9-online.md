# Karar gerekli: çevrimiçi kimlik (M9)

Sunucuyu çevrimiçi modda kabul etmek için istemci üç şeye ihtiyaç duyar; hiçbiri bugün yok ve bağımlılık seçimi
[AGENTS.md](../../AGENTS.md) uyarınca kullanıcının kararıdır.

1. **Şifreleme** (`Encryption` paketleri): RSA ile paylaşılan sır, AES-128/CFB8 akış, SHA-1 sunucu özeti. Seçenekler: RustCrypto (`rsa`, `aes`, `cfb8`, `sha1`) ya da sistem OpenSSL bağlaması.
2. **Oturum sunucusu isteği** (`joinServer`): HTTPS POST. Seçenekler: `ureq` + `rustls`, `reqwest`, ya da elle yazılmış TLS değil (önerilmez).
3. **Hesap belirteci**: Microsoft/Xbox/Minecraft oturum akışı (cihaz kodu), belirteç yenileme ve güvenli saklama. Gerçek hesap gerektirir; erişim yoksa `blocked` olarak kaydedilir.

Öneri: RustCrypto + `ureq` (küçük, bloklayıcı, mevcut iş parçacıklı ağ modeline uyar). Karar alınınca görevler: şifreleme (S–M), oturum isteği (S), hesap girişi (L).
