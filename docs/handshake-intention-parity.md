# P06.b — Handshake intention codec doğrulaması

Java oracle, 26.2 istemcisindeki `ClientIntentionPacket.STREAM_CODEC`'i `FriendlyByteBuf` üzerinde doğrudan çağırır. Rust adapter protocol sürümünü, UTF-16 host snapshot'ını, portu, intention id'sini, tüketimi, kalan byte'ları ve encode sırasında kısmi çıktıyı karşılaştırır.

18 özgün handshake senaryosu kapsandı: 8 yazma ve 10 okuma vakası. Yazma tarafında status/login/transfer, Unicode ve boş host, negatif protocol, port düşük 16-bit taşması, yazma sınırı ve prefix; okuma tarafında geçerli/trailing veri, kesik alanlar, port underflow, bilinmeyen intention, encoded/decoded host sınırı ve fazla uzun protocol VarInt denetlendi.

Java 25.0.4.1 / Linux x86_64 üzerinde odaklı capture kimliği `da81ea5da2917371b0f10d4f4512c42219ca6fb5f4c232c3fb2d83670e3e997e`; `verify-c9u61mo4`, debug `compare-uh7_sn_6` ve release `compare-lujyw4ez` raporları fark üretmedi. Handshake intention alt kümesi 18/18 eşleşti.

Güncel birleşik corpus 3476 senaryodur: `binary=164`, `math=2894`, `nbt=31`, `resources=44`, `timer=22`, `utf8=141`, `wire=180`. Birleşik capture kimliği `22b7eff1e5b7cdd30d0859f047d321b058410a257c2f9fc8cf4a38c747f0c1f6`; `verify-682kpa0b`, debug `compare-ezhc4kko` ve release `compare-6s5p7_y0` raporlarının tamamı geçti.

Bu sonuç P06.b'nin handshake intention codec sınırını doğrular. Connection state, status response, login challenge, compression/encryption ve vanilla 26.2 ile gerçek bağlantı akışı sonraki dilimlerdedir.
