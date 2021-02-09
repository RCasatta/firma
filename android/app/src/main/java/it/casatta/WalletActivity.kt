package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.json.Data
import kotlinx.android.synthetic.main.activity_wallet.*

class WalletActivity : ContextActivity() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())
    private val itemsAdapter = DescItemAdapter()
    private var walletDescriptor = ""

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_wallet)

        val walletString = intent.getStringExtra(C.WALLET)!!
        Log.d("WALLET", "${Network.TYPE} $walletString")
        val walletJson = mapper.readValue(walletString, Data.WalletJson::class.java)
        walletDescriptor = walletJson.descriptor
        val walletTitle = "wallet: ${walletJson.id.name}"
        title = walletTitle
        val qrContent = Data.StringEncoding(Data.Encoding.PLAIN, walletString)
        view_qr.setOnClickListener { QrActivity.comeHere(this, walletTitle, qrContent) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, walletJson.id.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }

        get_address.setOnClickListener { ListActivity.comeHere(this, ListActivity.ADDRESS_INDEX ) }

        val walletDir = "$filesDir/${Network.TYPE}/wallets/${walletJson.id.name}/"
        delete.setOnClickListener {
            C.showDeleteDialog(this, walletJson.id.name , walletDir)
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        itemsAdapter.list.add(DescItem("Descriptor main", walletJson.descriptor ))
        itemsAdapter.list.add(DescItem("Created at height", walletJson.created_at_height.toString() ))
        itemsAdapter.list.add(DescItem("Wallet json", mapper.writeValueAsString(walletJson) ))

        val signatureJson = exportSignature(walletJson.id.name)
        val signature = mapper.writeValueAsString(signatureJson)
        itemsAdapter.list.add(DescItem("Descriptor signature", signature))
        val signatureQrContent = Data.StringEncoding(Data.Encoding.PLAIN, signature)
        view_signature_qr.setOnClickListener { QrActivity.comeHere(this, walletTitle, signatureQrContent) }
    }

    override fun onActivityResult(
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ) {
        if (resultCode == Activity.RESULT_OK) {
            val index = data?.getStringExtra(C.RESULT)
            Log.i("WALLET", "index $index")
            val newIntent = Intent(this, AddressActivity::class.java)
            newIntent.putExtra(C.INDEX, "$index")
            newIntent.putExtra(C.DESCRIPTOR, walletDescriptor)
            startActivity(newIntent)
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }


}
