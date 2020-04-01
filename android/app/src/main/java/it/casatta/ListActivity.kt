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

        fun comeHere(from: Activity, what: Int, network: String) {
            val intent = Intent(from, ListActivity::class.java)

            intent.putExtra(C.WHAT, what)
            intent.putExtra(C.NETWORK, network)

            from.startActivityForResult(intent, what)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_list)
        val network = intent.getStringExtra(C.NETWORK)
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
                title = "Keys"
                updateKeys(network)
                item_new.setOnClickListener { view ->
                    comeHere(this, NEW_KEY, network)
                }
            }
            NEW_KEY -> {
                title = "New key"
                itemsAdapter.list.add(Item("random", null, null, emptyList()))
                itemsAdapter.list.add(Item("dice", null, null, emptyList()))
                itemsAdapter.list.add(Item("import xprv", null, null, emptyList()))
                itemsAdapter.list.add(Item("import bech32 seed", null, null, emptyList()))
                item_new.visibility = View.GONE
            }
            WALLETS -> {
                title = "Wallets"
                updateWallets(network)
                item_new.setOnClickListener { view ->
                    launchScan("Scan a Wallet")
                }
            }
            PSBTS -> {
                title = "PSBTs"
                updatePsbts(network)
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

    private fun updateKeys(network: String) {
        Log.d("LIST", "updateKeys $network")
        update(network, "keys")
        for (key in listOutput.keys) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(key);
            itemsAdapter.list.add(Item(key.key.name, key.key.fingerprint, details, key.public_qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updateWallets(network: String) {
        Log.d("LIST", "updateWallets $network")
        update(network, "wallets")
        for (wallet in listOutput.wallets) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(wallet);
            itemsAdapter.list.add(Item(wallet.wallet.name, wallet.wallet.fingerprints.toString(), details, wallet.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updatePsbts(network: String) {
        Log.d("LIST", "updatePsbts $network")
        update(network, "psbts")
        for (psbt in listOutput.psbts) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(psbt);
            itemsAdapter.list.add(Item(psbt.psbt.name, null, details, psbt.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun update(network: String, kind: String) {
        try {
            itemsAdapter.list.clear()
            listOutput = Rust().list(filesDir.toString(), network, kind)
        } catch (e: RustException) {
            Toast.makeText(this, e.toString(), Toast.LENGTH_LONG).show()
        }
    }

    override fun onCreateContextMenu(menu: ContextMenu?, v: View?, menuInfo: ContextMenu.ContextMenuInfo?) {
        super.onCreateContextMenu(menu, v, menuInfo)
        val inflater = menuInflater
        inflater.inflate(R.menu.menu_key, menu)
    }

    override fun onContextItemSelected(item: MenuItem?): Boolean {
        return when (item!!.itemId) {
            R.id.details ->{
                val key = itemsAdapter.list[itemsAdapter.position]
                val showText = TextView(this)
                showText.text = key.details
                showText.setTextIsSelectable(true)
                AlertDialog.Builder(this).setView(showText).create().show()
                return true
            }
            R.id.qr ->{
                val key = itemsAdapter.list[itemsAdapter.position]
                var qrs = emptyList<String>()
                when (intent.getIntExtra(C.WHAT, 0)) {
                    KEYS -> {
                        qrs = listOutput.keys.filter { it.key.name == key.name }.map { it.public_qr_files }.flatten()
                    }
                    WALLETS -> {
                        qrs = listOutput.wallets.filter { it.wallet.name == key.name }.map { it.qr_files }.flatten()
                    }
                    PSBTS -> {
                        qrs = listOutput.psbts.filter { it.psbt.name == key.name }.map { it.qr_files }.flatten()
                    }
                }
                QrActivity.comeHere(this, QrActivity.KEY,  qrs )
                return true
            }
            else -> super.onOptionsItemSelected(item)
        }
    }

    override fun onItemClick(item: Item) {
        val network = intent.getStringExtra(C.NETWORK)
        val what = intent.getIntExtra(C.WHAT, 0)
        Log.d("LOG","onItemClick $item.name $network $what")

        if (what == NEW_KEY) {
            keyNameDialog(item.name)
        } else {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, item.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }
    }

    private fun valueDialog(name: String, nature: String) {
        val valueEditText = EditText(this)

        val dialog: AlertDialog = AlertDialog.Builder(this)
            .setTitle("Insert $nature")
            .setView(valueEditText)
            .setPositiveButton("Ok") { dialog, which ->
                val network = intent.getStringExtra(C.NETWORK)
                val valueEditText = valueEditText.text.toString()
                try {
                    Rust().restore(filesDir.toString(), network, name, nature, valueEditText)
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
                val network = intent.getStringExtra(C.NETWORK)
                val keyName = keyEditText.text.toString()
                val keyFile = File("$filesDir/$network/keys/$keyName/PRIVATE.json")
                if (keyFile.exists()) {
                    Toast.makeText(this, "This key already exist", Toast.LENGTH_LONG).show()
                } else {
                    when (what)  {
                        "random" -> {
                            Rust().random(filesDir.toString(), network, keyName)
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
        val network = intent.getStringExtra(C.NETWORK)
        Log.d("MAIN", "saveWallet " + content)
        try {
            val json = mapper.readValue(content, Rust.WalletJson::class.java)
            val name = json.name
            val networkDir = File(filesDir, network)
            val wallets = File(networkDir, "wallets")
            val wallet = File(wallets, name)
            if (!wallet.exists()) {
                //TODO should save through API, so that qr code are created
                wallet.mkdirs()
                val desc = File(wallet, "descriptor.json")
                Log.d("MAIN", "saveWallet path " + desc)
                desc.appendText(content)
                Rust().create_qrs(desc.toString(), network)
                updateWallets(network)
            } else {
                Toast.makeText(this, "This wallet already exist", Toast.LENGTH_LONG).show()
            }
        } catch (e: Exception) {
            Toast.makeText(this, "This is not a wallet", Toast.LENGTH_LONG).show()
        }
    }

    fun savePsbt(content: String) {
        val network = intent.getStringExtra(C.NETWORK)
        Log.d("MAIN", "savePsbt " + content)
        try {
            val json = mapper.readValue(content, Rust.PsbtJson::class.java)
            val name = json.name
            val networkDir = File(filesDir, network)
            val psbts = File(networkDir, "psbts")
            val psbt = File(psbts, name)
            if (!psbt.exists()) {
                //TODO should save through API, so that qr code are created
                psbt.mkdirs()
                val desc = File(psbt, "psbt.json")
                Log.d("MAIN", "savePsbt path " + desc)
                desc.appendText(content)
                Rust().create_qrs(desc.toString(), network)
                updatePsbts(network)
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
                        val network = intent.getStringExtra(C.NETWORK)
                        val hexResult = Rust().merge_qrs(filesDir.toString(), network, this.rawBytes)
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
            val network = intent.getStringExtra(C.NETWORK)
            updateKeys(network)
        } else {
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    fun hexToByte(hexString: String): Byte {
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

    fun decodeHexString(hexString: String): ByteArray? {
        if (hexString.length % 2 === 1) {
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

data class Item(val name: String, val description: String?, val details: String?, val qrs: List<String>): Serializable

class ItemsAdapter() : RecyclerView.Adapter<ItemHolder>(),
    View.OnCreateContextMenuListener {

    val list: ArrayList<Item> = ArrayList()
    var listener: ItemGesture? = null
    var position = -1

    override fun getItemCount():Int{
        return list.size
    }

    override fun onBindViewHolder(holder: ItemHolder, position: Int) {
        var tx = list[position]
        holder.update(tx)
        holder.itemView.setOnLongClickListener {
            this.position = position
            false
        }
        holder.itemView.setOnClickListener {
            listener?.onItemClick(tx)
        }
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ItemHolder {
        var item = LayoutInflater.from(parent.context).inflate(R.layout.key_item, parent, false)
        item.setOnCreateContextMenuListener(this)
        return ItemHolder(item)
    }

    interface ItemGesture {
        fun onItemClick(item: Item)
    }

    override fun onCreateContextMenu(
        menu: ContextMenu?,
        v: View?,
        menuInfo: ContextMenu.ContextMenuInfo?
    ) {
        // context menu doesn't work by default on RecyclerView because it's a ViewGroup and not a View,
        // calling item.setOnCreateContextMenuListener(this) apparently solve this even if this method is
        // empty, onCreateContextMenu from ListActivity is called and the inflater is available there
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


