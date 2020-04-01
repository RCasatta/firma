package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.widget.CompoundButton
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import kotlinx.android.synthetic.main.activity_main.*

class MainActivity : AppCompatActivity() {

    companion object {
        const val EMPTY_KEY = "Select key"
        const val EMPTY_WALLET = "Select wallet"
        const val EMPTY_PSBT = "Select PSBT"
        const val BITCOIN_NETWORK = "bitcoin"
        const val TESTNET_NETWORK = "testnet"
    }


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        network_text.text = TESTNET_NETWORK
        network_switch.isChecked = false
        reset_select()

        network_switch.setOnCheckedChangeListener { compoundButton: CompoundButton, b: Boolean ->
            Log.d("MAIN", "changed $b")
            network_text.text =  if (b) BITCOIN_NETWORK else TESTNET_NETWORK
            reset_select()
        }

        key_button.setOnClickListener {view ->
            ListActivity.comeHere(this, ListActivity.KEYS, network())
        }

        wallet_button.setOnClickListener {view ->
            ListActivity.comeHere(this, ListActivity.WALLETS, network())
        }

        psbt_button.setOnClickListener {view ->
            ListActivity.comeHere(this, ListActivity.PSBTS, network())
        }

        sign_button.isEnabled = true
        sign_button.setOnClickListener { view->
            if (key_text.text == EMPTY_KEY || wallet_text.text == EMPTY_WALLET  || psbt_text.text == EMPTY_PSBT) {
                Toast.makeText(this, "Select key, wallet and psbt", Toast.LENGTH_LONG).show()
            } else {
                val keyFile = "$filesDir/${network()}/keys/${key_text.text}/PRIVATE.json"
                val walletFile = "$filesDir/${network()}/wallets/${wallet_text.text}/descriptor.json"
                val psbtFile = "$filesDir/${network()}/psbts/${psbt_text.text}/psbt.json"
                val result = Rust().sign(filesDir.toString(), network(), keyFile,walletFile, psbtFile )
                AlertDialog.Builder(this).setMessage(result.toString()).create().show()
            }
        }

    }

    private fun reset_select() {
        key_text.text = EMPTY_KEY
        wallet_text.text = EMPTY_WALLET
        psbt_text.text = EMPTY_PSBT
    }

    fun network() : String {
        if (network_switch.isChecked)
            return BITCOIN_NETWORK
        else
            return TESTNET_NETWORK
    }

    override fun onActivityResult(
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ) {
        if (resultCode == Activity.RESULT_OK) {
            val result = data?.getStringExtra(C.RESULT)
            if (requestCode == ListActivity.KEYS) {
                key_text.text = result
            } else if (requestCode == ListActivity.WALLETS) {
                wallet_text.text = result
            } else if (requestCode == ListActivity.PSBTS) {
                psbt_text.text = result
            }
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }
}

fun ByteArray.toHexString() = joinToString("") { "%02x".format(it) }
