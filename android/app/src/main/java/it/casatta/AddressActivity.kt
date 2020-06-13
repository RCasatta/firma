package it.casatta

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import kotlinx.android.synthetic.main.activity_address.*

class AddressActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_address)

        val index = intent.getStringExtra(C.INDEX)!!.toInt()
        val descriptor = intent.getStringExtra(C.DESCRIPTOR)!!
        Log.i("ADDRESS", "$index $descriptor")

        val addressData = Rust().deriveAddress(descriptor, index)

        address.text = addressData.address
        path.text = addressData.path
    }

}
