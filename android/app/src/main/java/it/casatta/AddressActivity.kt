package it.casatta

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AlertDialog
import kotlinx.android.synthetic.main.activity_address.*

class AddressActivity : ContextActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_address)

        val index = intent.getStringExtra(C.INDEX)!!.toInt()
        val descriptor = intent.getStringExtra(C.DESCRIPTOR)!!
        Log.i("ADDRESS", "$index $descriptor")

        val addressData = Rust().deriveAddress(context(), descriptor, index)

        address.text = addressData.address
        path.text = addressData.path

        info_address.setOnClickListener { AlertDialog.Builder(this).setTitle("Attention").setMessage("Double check this address ant the path with the ones generated with the online tool. This prevents an attacker replacing the address generated with the online tool with an address he owns.").create().show() }
    }

}
