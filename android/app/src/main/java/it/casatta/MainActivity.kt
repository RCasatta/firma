package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import kotlinx.android.synthetic.main.activity_main.*

class MainActivity : AppCompatActivity() {

    companion object {
        init {
            System.loadLibrary("firma")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        title = "Firma (${Network.TYPE})"

        key_button.setOnClickListener {
            ListActivity.comeHere(this, ListActivity.KEYS)
        }

        wallet_button.setOnClickListener {
            ListActivity.comeHere(this, ListActivity.WALLETS)
        }

        psbt_button.setOnClickListener {
            ListActivity.comeHere(this, ListActivity.PSBTS)
        }

        sign_button.isEnabled = true
        sign_button.setOnClickListener {
            if (key_text.text == getString(R.string.select_key) || wallet_text.text == getString(R.string.select_wallet)  || psbt_text.text == getString(R.string.select_transaction)) {
                C.showMessageDialog(this, R.string.select_all)
            } else {
                val keyFile = "$filesDir/${Network.TYPE}/keys/${key_text.text}/PRIVATE.json"
                val walletFile = "$filesDir/${Network.TYPE}/wallets/${wallet_text.text}/descriptor.json"
                val psbtFile = "$filesDir/${Network.TYPE}/psbts/${psbt_text.text}/psbt.json"
                try {
                    val result = Rust().sign(filesDir.toString(), keyFile, walletFile, psbtFile, EncryptionKey.get(applicationContext))
                    if (result.info.contains(getString(R.string.added_signatures))) {
                        AlertDialog.Builder(this).setMessage(R.string.added_signatures).create().show()
                    } else {
                        AlertDialog.Builder(this).setMessage(R.string.no_signatures_added).create().show()
                    }
                } catch (e: RustException) {
                    C.showMessageDialog(this, e.message?:"Null")
                }
            }
        }

        C.showMessageIfInIntent(this, intent)
    }

    override fun onActivityResult(
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ) {
        if (resultCode == Activity.RESULT_OK) {
            val result = data?.getStringExtra(C.RESULT)
            when (requestCode) {
                ListActivity.KEYS -> key_text.text = result
                ListActivity.WALLETS -> wallet_text.text = result
                ListActivity.PSBTS -> psbt_text.text = result
            }
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }
}

fun ByteArray.toHexString() = joinToString("") { "%02x".format(it) }
//fun String.hexStringToByteArray() = ByteArray(this.length / 2) { this.substring(it * 2, it * 2 + 2).toInt(16).toByte() }

