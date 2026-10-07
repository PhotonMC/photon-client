# P04.b — Pack kaynak önceliği sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 MultiPackResourceManager ve FallbackResourceManager için temel client-resource pack sırasını kapsar.

Pack listesi verilen sırayla namespace yöneticisine eklenir. Aynı identifier birden fazla pack'te varsa kaynak araması listenin sonundan yapılır ve son eşleşme seçilir. Kaynak stack, eşleşen pack'leri düşük öncelikliden yüksek öncelikliye doğru döndürür. Hiçbir pack'te bulunmayan identifier boş sonuç verir.

Rust üretim modülü pack kimliklerini ve sentetik kaynak byte'larını verilen sırada tutar; tekil lookup son pack'ten ilk eşleşmeyi, stack lookup ise giriş sırasını döndürür. Pack metadata/filter/overlay, namespace keşfi ve reload davranışı bu sözleşmeye dahil değildir.


## Client-parity doğrulaması

Java oracle, geçerli client-resource dizinleri içeren sentetik PathPackResources örneklerini MultiPackResourceManager içine verilen sırada yerleştirir. Her identifier için seçilen pack, seçilen byte dizisi ve getResourceStack kaynak sırası karşılaştırılır.

Kapsanan durumlar:

- iki pack'te ortak identifier için son pack'in seçilmesi;
- yalnızca alt veya üst pack'te bulunan kaynaklar;
- eksik identifier;
- üç katmanlı stack ve dizin altındaki kaynak sorgusu.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- altı özgün senaryo iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `3e4bd88ce740d6f091bf439ae63549f739f16d2e3953ca96811524184be91321`;
- Java `verify` raporu `verify-xcoyk0rm`;
- Rust debug karşılaştırması `compare-ani40avt`, release karşılaştırması `compare-06h24203` ve her ikisi de fark üretmedi.

Pack filter, metadata, overlay ve reload sonraki P04 dilimleridir.
