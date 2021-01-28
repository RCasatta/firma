package it.casatta

import android.content.Context
import android.util.Log
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import it.casatta.json.Data
import it.casatta.json.toHexString
import java.security.SecureRandom


class EncryptionKey {

    companion object {
        @Synchronized
        fun get(context: Context): Data.StringEncoding {
            Log.i("EncryptionKey", "get")
            val masterKeyAlias =
                MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build();
            val sharedPreferences = EncryptedSharedPreferences.create(
                context,
                "secret_shared_prefs",
                masterKeyAlias,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
            )
            val encryptionKeyHex = sharedPreferences.getString("encryption_key_hex", "")!!
            if (encryptionKeyHex.isEmpty()) {
                val newEncryptionKeyHex = create()
                sharedPreferences.edit().putString("encryption_key_hex", newEncryptionKeyHex).commit()
                return get(context)
            }

            return Data.StringEncoding( Data.Encoding.HEX, encryptionKeyHex)
        }

        private fun create(): String {
            Log.i("EncryptionKey", "create")
            val random = SecureRandom()
            val bytes = ByteArray(32)
            random.nextBytes(bytes)
            return bytes.toHexString()
        }
    }

}