package it.casatta

import android.app.Activity
import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import android.view.View
import androidx.recyclerview.widget.LinearLayoutManager
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import kotlinx.android.synthetic.main.activity_key.*
import kotlinx.android.synthetic.main.activity_key.delete
import kotlinx.android.synthetic.main.activity_key.items
import kotlinx.android.synthetic.main.activity_key.select
import kotlinx.android.synthetic.main.activity_key.view_qr
import kotlinx.android.synthetic.main.activity_wallet.*

class KeyActivity : AppCompatActivity() {
    private val mapper: ObjectMapper = ObjectMapper().registerModule(KotlinModule())
    private val itemsAdapter = DescItemAdapter()
    private val hiddenItemsAdapter = DescItemAdapter()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_key)

        val keyString = intent.getStringExtra(C.KEY)
        Log.d("KEY", "${Network.TYPE} $keyString")
        val keyJson = mapper.readValue(keyString, Rust.MasterKeyOutput::class.java)
        val keyTitle = "key: ${keyJson.key.name}"
        title = keyTitle

        view_qr.setOnClickListener { QrActivity.comeHere(this, keyTitle, keyJson.public_qr_files ) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, keyJson.key.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }
        show_private.setOnClickListener {
            if (hidden_items.visibility == View.GONE) {
                hidden_items.visibility = View.VISIBLE
            } else {
                hidden_items.visibility = View.GONE
            }
        }
        val keyDir = "$filesDir/${Network.TYPE}/keys/${keyJson.key.name}/"
        delete.setOnClickListener {
            C.showDeleteDialog(this, keyJson.key.name , keyDir)
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        itemsAdapter.list.add(DescItem("Fingerprint", keyJson.key.fingerprint))
        itemsAdapter.list.add(DescItem("Xpub", keyJson.key.xpub))

        hidden_items.layoutManager = LinearLayoutManager(this)
        hidden_items.adapter = hiddenItemsAdapter
        hiddenItemsAdapter.list.add(DescItem("Xpriv", keyJson.key.xprv))

        if (keyJson.key.seed != null) {
            hiddenItemsAdapter.list.add(DescItem("Seed", keyJson.key.seed.bech32))
        }
    }
}
