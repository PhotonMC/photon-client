# P06.b — Login completion packet sözleşmesi

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `ServerboundLoginAcknowledgedPacket.STREAM_CODEC` status request ile aynı unit codec davranışına sahiptir: encode hiçbir bayt yazmaz, decode input'u tüketmeden singleton döndürür.

`ClientboundLoginFinishedPacket` önce GameProfile UUID'sini, 16 UTF-16 birimlik adı ve en fazla 16 property kaydını, sonra session UUID'sini taşır. Her property adı 64, değeri 32767, nullable signature değeri 1024 UTF-16 birimiyle `Utf8String` sınırlarına tabidir. Property kaydı name, value ve signature-present boolean sırasındadır; signature varsa boolean'dan sonra string gelir.

Rust tarafı GameProfile alanlarını UTF-16 birimleriyle, UUID'leri `binary::Uuid` ile ve property kayıtlarını giriş sırasındaki `Vec<ProfileProperty>` ile taşır. Count sınırı count VarInt'inden sonra, UUID ve önceki alanlar tüketilmişken uygulanır. Kısa UUID/alan okumaları Java'nın alan sırasını ve kısmi tüketimini korur.

Bu dilim packet payload codec'leriyle sınırlıdır. Authentication, encryption, login listener/state transition ve configuration phase packet'leri sonraki dilimlerdir.
