# P06.a — VarInt21 frame sınırı doğrulaması

Java oracle, 26.2 istemcisindeki `Varint21LengthFieldPrepender` ve `Varint21FrameDecoder` sınıflarını gerçek Netty `ByteBuf` üzerinde doğrudan çağırır. Rust adapter aynı gövde üretimini, parçalı input çağrılarını, frame byte'larını, tüketimi, kalan veriyi ve hata kategorisini karşılaştırır.

18 özgün wire senaryosu kapsandı: 8 prepender ve 10 decoder vakası. Prepender tarafında boş gövde, 1/2/3 bayt başlık eşikleri, 21-bit üst sınır, fazla büyük gövde ve reader offset'i; decoder tarafında parçalı header/body, eksik input, sıfır ve geniş header hataları, tek çağrıda trailing veri, ardışık frame'ler ve üç baytlık uzunluk denetlendi.

Java 25.0.4.1 / Linux x86_64 üzerinde odaklı referans capture kimliği `58a0c5de86f41490de6fa78d7bf4bb2c8726b79fa77a34cd221e8bb6c2fe3259`; `verify-ss2rh4bc`, debug `compare-i2ht8jbn` ve release `compare-5nhdbu91` raporları fark üretmedi. Wire ailesi 162/162 eşleşti.

Güncel birleşik corpus 3458 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=162`. Birleşik capture kimliği `7c3d152f0b70c212560e76201ea0351c26e946b995122d1f12aa001072ef0bac`; `verify-nh4zstxq`, debug `compare-52b4dvxe` ve release `compare-7i6v5xnv` raporlarının tamamı geçti.

Bu sonuç P06.a'nın headless frame sınırını doğrular. Socket bağlantısı, packet state machine, compression/encryption, login ve Pumpkin ile gerçek oyun oturumu sonraki P06/P07 dilimlerindedir.
