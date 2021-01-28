package it.casatta

import androidx.appcompat.app.AppCompatActivity

import it.casatta.json.Data

open class ContextActivity : AppCompatActivity() {
    fun context(): Data.Context {
        return Data.Context(filesDir.toString(), Network.TYPE, EncryptionKey.get(applicationContext))
    }
}