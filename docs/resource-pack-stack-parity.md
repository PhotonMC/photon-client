# P04.b — Pack kaynak önceliği doğrulaması

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
