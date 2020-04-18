package it.casatta

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.*
import android.widget.EditText
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.DividerItemDecoration
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import com.google.zxing.integration.android.IntentIntegrator
import kotlinx.android.synthetic.main.activity_list.*
import java.io.File
import java.io.Serializable

class ListActivity : AppCompatActivity() , ItemsAdapter.ItemGesture {
    val itemsAdapter = ItemsAdapter()
    var listOutput = Rust.ListOutput( emptyList(),  emptyList(),  emptyList())
    val mapper = ObjectMapper().registerModule(KotlinModule())

    companion object {
        const val KEYS = 1
        const val WALLETS = 2
        const val PSBTS = 3
        const val NEW_KEY = 4
        const val PSBT = 5
        const val WALLET = 6
        const val KEY = 7

        fun comeHere(from: Activity, what: Int, network: String) {
            val newIntent = Intent(from, ListActivity::class.java)

            newIntent.putExtra(C.WHAT, what)
            newIntent.putExtra(C.NETWORK, network)

            from.startActivityForResult(newIntent, what)
        }

        var Intent.network: String?
            get() = getStringExtra(C.NETWORK)
            set(message) {
                putExtra(C.NETWORK, message!!)
            }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_list)
        val recyclerView = findViewById<RecyclerView>(R.id.items_list)

        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.addItemDecoration(
            DividerItemDecoration(
                recyclerView.context,
                DividerItemDecoration.VERTICAL
            )
        )
        itemsAdapter.listener = this
        recyclerView.adapter = itemsAdapter
        when (intent.getIntExtra(C.WHAT, 0)) {
            KEYS -> {
                title = "${intent.network} keys"
                updateKeys()
                item_new.setOnClickListener { view ->
                    comeHere(this, NEW_KEY, intent.network!!)
                }
            }
            NEW_KEY -> {
                title = "${intent.network} new key"
                itemsAdapter.list.add(Item("random", null, null, emptyList()))
                itemsAdapter.list.add(Item("dice", null, null, emptyList()))
                itemsAdapter.list.add(Item("import xprv", null, null, emptyList()))
                itemsAdapter.list.add(Item("import bech32 seed", null, null, emptyList()))
                item_new.visibility = View.GONE
            }
            WALLETS -> {
                title = "${intent.network} wallets"
                updateWallets()
                item_new.setOnClickListener { view ->
                    launchScan("Scan a Wallet")
                }
            }
            PSBTS -> {
                title = "${intent.network} PSBTs"
                updatePsbts()
                item_new.setOnClickListener { view ->
                    launchScan("Scan a PSBT")
                }
            }
            else -> {
                Log.d("LIST", "others" )
            }
        }

        registerForContextMenu(recyclerView)
    }

    private fun launchScan(title: String) {
        val integrator = IntentIntegrator(this)
        integrator.setOrientationLocked(false)
        integrator.setDesiredBarcodeFormats(IntentIntegrator.QR_CODE);
        integrator.setPrompt(title);
        integrator.initiateScan()
    }

    private fun updateKeys() {
        update("keys")
        for (key in listOutput.keys) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(key);
            itemsAdapter.list.add(Item(key.key.name, key.key.fingerprint, details, key.public_qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updateWallets() {
        update("wallets")
        for (wallet in listOutput.wallets) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(wallet);
            itemsAdapter.list.add(Item(wallet.wallet.name, wallet.wallet.fingerprints.toString(), details, wallet.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updatePsbts() {
        update("psbts")
        for (psbt in listOutput.psbts) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(psbt);
            itemsAdapter.list.add(Item(psbt.psbt.name, null, details, psbt.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun update(kind: String) {
        try {
            itemsAdapter.list.clear()
            listOutput = Rust().list(filesDir.toString(), intent.network!!, kind)
        } catch (e: RustException) {
            Toast.makeText(this, e.toString(), Toast.LENGTH_LONG).show()
        }
    }

    override fun onItemLongClick(item: Item) {
        val returnIntent = Intent()
        returnIntent.putExtra(C.RESULT, item.name)
        setResult(Activity.RESULT_OK, returnIntent)
        finish()
    }

    override fun onItemClick(item: Item) {
        val what = intent.getIntExtra(C.WHAT, 0)
        Log.d("LOG","onItemClick $item.name ${intent.network!!} $what")

        when(what) {
            NEW_KEY -> {
                keyNameDialog(item.name)
            }
            KEYS -> {
                val newIntent = Intent(this, KeyActivity::class.java)
                newIntent.putExtra(C.NETWORK, intent.network)
                newIntent.putExtra(C.KEY, item.json)
                startActivityForResult(newIntent, KEY)
            }
            WALLETS -> {
                val newIntent = Intent(this, WalletActivity::class.java)
                newIntent.putExtra(C.NETWORK, intent.network)
                newIntent.putExtra(C.WALLET, item.json)
                startActivityForResult(newIntent, WALLET)
            }
            PSBTS -> {
                val newIntent = Intent(this, PSBTActivity::class.java)
                newIntent.putExtra(C.NETWORK, intent.network)
                newIntent.putExtra(C.PSBT, item.json)
                startActivityForResult(newIntent, PSBT)
            }
            else -> {
                Log.w("LIST", "not mapped")
            }
        }
    }

    private fun valueDialog(name: String, nature: String) {
        val valueEditText = EditText(this)

        val dialog: AlertDialog = AlertDialog.Builder(this)
            .setTitle("Insert $nature")
            .setView(valueEditText)
            .setPositiveButton("Ok") { dialog, which ->

                val valueEditText = valueEditText.text.toString()
                try {
                    Rust().restore(filesDir.toString(), intent.network!!, name, nature, valueEditText)
                    setResult(Activity.RESULT_OK, Intent())
                } catch (e: RustException) {
                    Toast.makeText(this, e.toString(), Toast.LENGTH_LONG).show()
                    setResult(Activity.RESULT_CANCELED, Intent())
                }
                finish()

            }
            .setNegativeButton("Cancel", null)
            .create()
        dialog.show()
    }

    private fun keyNameDialog(what: String) {
        val keyEditText = EditText(this)

        val dialog: AlertDialog = AlertDialog.Builder(this)
            .setTitle("New key")
            .setMessage("Give this key a unique name.")
            .setView(keyEditText)
            .setPositiveButton("Ok") { dialog, which ->
                val keyName = keyEditText.text.toString()
                val keyFile = File("$filesDir/${intent.network!!}/keys/$keyName/PRIVATE.json")
                if (keyFile.exists()) {
                    Toast.makeText(this, "This key already exist", Toast.LENGTH_LONG).show()
                } else {
                    when (what)  {
                        "random" -> {
                            Rust().random(filesDir.toString(), intent.network!!, keyName)
                            setResult(Activity.RESULT_OK, Intent())
                            finish()
                        }
                        "dice" -> {
                            Toast.makeText(this, "Not yet implemented", Toast.LENGTH_LONG).show()
                        }
                        "import xprv" -> {
                            valueDialog(keyName, "Xprv" )
                        }
                        "import bech32 seed" -> {
                            valueDialog(keyName, "Bech32Seed")
                        }
                    }
                }
            }
            .setNegativeButton("Cancel", null)
            .create()
        dialog.show()
    }

    fun saveWallet(content: String) {
        Log.d("MAIN", "saveWallet " + content)
        try {
            val json = mapper.readValue(content, Rust.WalletJson::class.java)
            val name = json.name
            val networkDir = File(filesDir, intent.network!!)
            val wallets = File(networkDir, "wallets")
            val wallet = File(wallets, name)
            if (!wallet.exists()) {
                wallet.mkdirs()
                val desc = File(wallet, "descriptor.json")
                Log.d("MAIN", "saveWallet path " + desc)
                desc.writeText(content)
                Rust().create_qrs(desc.toString(), intent.network!!)
                updateWallets()
            } else {
                Toast.makeText(this, "This wallet already exist", Toast.LENGTH_LONG).show()
            }
        } catch (e: Exception) {
            Toast.makeText(this, "This is not a wallet", Toast.LENGTH_LONG).show()
        }
    }

    fun savePsbt(content: String) {
        Log.d("MAIN", "savePsbt " + content)
        try {
            val json = mapper.readValue(content, Rust.PsbtJson::class.java)
            val name = json.name
            val networkDir = File(filesDir, intent.network!!)
            val psbts = File(networkDir, "psbts")
            val psbt = File(psbts, name)
            if (!psbt.exists()) {
                psbt.mkdirs()
                val desc = File(psbt, "psbt.json")
                Log.d("MAIN", "savePsbt path " + desc)
                desc.writeText(content)
                Rust().create_qrs(desc.toString(), intent.network!!)
                updatePsbts()
            } else {
                Toast.makeText(this, "This psbt already exist", Toast.LENGTH_LONG).show()
            }
        } catch (e: Exception) {
            Toast.makeText(this, "This is not a psbt", Toast.LENGTH_LONG).show()
        }
    }

    var rawBytes: ArrayList<String> = ArrayList()
    override fun onActivityResult(
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ) {
        val result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (result != null) {
            if (result.contents == null) {
                rawBytes = ArrayList()
                Toast.makeText(this, "Cancelled", Toast.LENGTH_LONG).show()
            } else {
                val hexString = result.rawBytes.toHexString()
                this.rawBytes.add(hexString)
                if (hexString.startsWith("3")) {
                    try {
                        val hexResult = Rust().merge_qrs(filesDir.toString(), intent.network!!, this.rawBytes)
                        rawBytes = ArrayList()
                        Log.d("MAIN", "qr complete: $result")
                        val bytes = decodeHexString(hexResult)
                        when (intent.getIntExtra(C.WHAT, 0)) {
                            WALLETS -> {
                                saveWallet(bytes!!.toString(Charsets.UTF_8))
                            }
                            PSBTS -> {
                                savePsbt(bytes!!.toString(Charsets.UTF_8))
                            }
                        }
                    } catch (e: RustException) {
                        launchScan("Next")
                    }
                } else {
                    when (intent.getIntExtra(C.WHAT, 0)) {
                        WALLETS -> {
                            saveWallet(result.contents)
                        }
                        PSBTS -> {
                            savePsbt(result.contents)
                        }
                    }
                }
            }
        } else if (requestCode == NEW_KEY && resultCode == Activity.RESULT_OK) {
            updateKeys()
        }  else if (requestCode in arrayOf(PSBT,WALLET,KEY) && resultCode == Activity.RESULT_OK) {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, data!!.getStringExtra(C.RESULT))
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    private fun hexToByte(hexString: String): Byte {
        val firstDigit = toDigit(hexString[0])
        val secondDigit = toDigit(hexString[1])
        return ((firstDigit shl 4) + secondDigit).toByte()
    }

    private fun toDigit(hexChar: Char): Int {
        val digit: Int = Character.digit(hexChar, 16)
        if (digit == -1) {
            throw IllegalArgumentException(
                    "Invalid Hexadecimal Character: $hexChar")
        }
        return digit
    }

    private fun decodeHexString(hexString: String): ByteArray? {
        if (hexString.length % 2 == 1) {
            throw IllegalArgumentException(
                    "Invalid hexadecimal String supplied.")
        }
        val bytes = ByteArray(hexString.length / 2)
        var i = 0
        while (i < hexString.length ) {
            bytes[i / 2] = hexToByte(hexString.substring(i, i + 2))
            i += 2
        }
        return bytes
    }
}

data class Item(val name: String, val description: String?, val json: String?, val qrs: List<String>): Serializable

class ItemsAdapter() : RecyclerView.Adapter<ItemHolder>() {

    val list: ArrayList<Item> = ArrayList()
    var listener: ItemGesture? = null

    override fun getItemCount():Int{
        return list.size
    }

    override fun onBindViewHolder(holder: ItemHolder, position: Int) {
        var tx = list[position]
        holder.update(tx)
        holder.itemView.setOnClickListener {
            listener?.onItemClick(tx)
        }
        holder.itemView.setOnLongClickListener {
            listener?.onItemLongClick(tx)
            true
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ItemHolder {
        var item = LayoutInflater.from(parent.context).inflate(R.layout.key_item, parent, false)
        return ItemHolder(item)
    }

    interface ItemGesture {
        fun onItemClick(item: Item)
        fun onItemLongClick(item: Item)
    }
}

class ItemHolder(itemView: View): RecyclerView.ViewHolder(itemView) {
    private val name = itemView.findViewById<TextView>(R.id.name)
    private val description = itemView.findViewById<TextView>(R.id.description)

    fun update(item: Item) {
        name.text = item.name
        if (item.description != null) {
            description.visibility = View.VISIBLE
            description.text = item.description
        } else {
            description.visibility = View.GONE
        }
    }
}
