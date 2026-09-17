# kotlinx.serialization — keep generated serializers for DTOs.
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.AnnotationsKt
-keepclassmembers class kotlinx.serialization.json.** { *** Companion; }
-keepclasseswithmembers class kotlinx.serialization.json.** { kotlinx.serialization.KSerializer serializer(...); }
-keep,includedescriptorclasses class me.siwi.irsclaw.**$$serializer { *; }
-keepclassmembers class me.siwi.irsclaw.** { *** Companion; }
-keepclasseswithmembers class me.siwi.irsclaw.** { kotlinx.serialization.KSerializer serializer(...); }

# Markwon reads HTML tags / classes reflectively in some plugins.
-dontwarn io.noties.markwon.**
