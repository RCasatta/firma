package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.View
import androidx.recyclerview.widget.LinearLayoutManager
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.json.Data.*;
import kotlinx.android.synthetic.main.activity_key.*
import kotlinx.android.synthetic.main.activity_key.delete
import kotlinx.android.synthetic.main.activity_key.items
import kotlinx.android.synthetic.main.activity_key.select
import kotlinx.android.synthetic.main.activity_key.view_qr

class KeyActivity : ContextActivity() {
    private val mapper: ObjectMapper = ObjectMapper().registerModule(KotlinModule())
    private val itemsAdapter = DescItemAdapter()
    private val hiddenItemsAdapter = DescItemAdapter()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_key)

        val keyString = intent.getStringExtra(C.KEY)
        val keyJson = mapper.readValue(keyString, MasterSecret::class.java)
        val keyTitle = "key: ${keyJson.id.name}"
        title = keyTitle

        val descJson = exportDescriptorPublicKey(keyJson.id.name)
        val qrContent = StringEncoding(Encoding.PLAIN, descJson.desc_pub_key)
        view_qr.setOnClickListener { QrActivity.comeHere(this, keyTitle, qrContent) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, keyJson.id.name)
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
        val keyDir = "$filesDir/${Network.TYPE}/keys/${keyJson.id.name}/"
        delete.setOnClickListener {
            C.showDeleteDialog(this, keyJson.id.name , keyDir)
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        //itemsAdapter.list.add(DescItem("Fingerprint", keyJson.fingerprint))
        itemsAdapter.list.add(DescItem("Descriptor Public Key", descJson.desc_pub_key))

        hidden_items.layoutManager = LinearLayoutManager(this)
        hidden_items.adapter = hiddenItemsAdapter
        hiddenItemsAdapter.list.add(DescItem("Xpriv", keyJson.key))

        if (keyJson.dice != null) {
            hiddenItemsAdapter.list.add(DescItem("Faces", keyJson.dice.faces.toString() ))
            hiddenItemsAdapter.list.add(DescItem("Launches", keyJson.dice.launches ))
            hiddenItemsAdapter.list.add(DescItem("Value", keyJson.dice.value ))
        }
    }
}
