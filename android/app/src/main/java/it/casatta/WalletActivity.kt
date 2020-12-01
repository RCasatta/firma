package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import kotlinx.android.synthetic.main.activity_wallet.*

class WalletActivity : AppCompatActivity() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())
    private val itemsAdapter = DescItemAdapter()
    private var walletDescriptor = ""

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_wallet)

        val walletString = intent.getStringExtra(C.WALLET)
        Log.d("WALLET", "${Network.TYPE} $walletString")
        val walletJson = mapper.readValue(walletString, Rust.CreateWalletOutput::class.java)
        walletDescriptor = walletJson.wallet.descriptor_main
        val walletTitle = "wallet: ${walletJson.wallet.name}"
        title = walletTitle

        view_qr.setOnClickListener { QrActivity.comeHere(this, walletTitle, walletJson.qr_files ) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, walletJson.wallet.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }

        get_address.setOnClickListener { ListActivity.comeHere(this, ListActivity.ADDRESS_INDEX ) }

        val walletDir = "$filesDir/${Network.TYPE}/wallets/${walletJson.wallet.name}/"
        delete.setOnClickListener {
            C.showDeleteDialog(this, walletJson.wallet.name , walletDir)
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        itemsAdapter.list.add(DescItem("Fingerprints", walletJson.wallet.fingerprints.toString() ))
        itemsAdapter.list.add(DescItem("Descriptor main", walletJson.wallet.descriptor_main ))
        itemsAdapter.list.add(DescItem("Descriptor change", walletJson.wallet.descriptor_change ))
        itemsAdapter.list.add(DescItem("Required sig", walletJson.wallet.required_sig.toString() ))
        itemsAdapter.list.add(DescItem("Created at height", walletJson.wallet.created_at_height.toString() ))
        itemsAdapter.list.add(DescItem("Wallet json", mapper.writeValueAsString(walletJson.wallet) ))
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
