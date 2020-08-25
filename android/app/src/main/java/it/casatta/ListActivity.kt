package it.casatta

import android.app.Activity
import android.content.*
import android.os.Bundle
import android.text.InputType
import android.util.Log
import android.view.*
import android.widget.EditText
import android.widget.TextView
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

    private val itemsAdapter = ItemsAdapter()
    private var listOutput = Rust.ListOutput( emptyList(),  emptyList(),  emptyList())
    private val mapper: ObjectMapper = ObjectMapper().registerModule(KotlinModule())
    private var rawHexes: ArrayList<String> = ArrayList()
    private var diceLaunches: ArrayList<Int> = ArrayList()
    private var faces: Int = 0
    private var keyName: String? = null
    private val PERFECT_SOLID_FACES = listOf(2, 4, 6, 8, 12, 20)
    private val LAUNCHES_FOR_256_BIT = listOf(256, 128, 99, 85, 71, 59)

    companion object {
        const val KEYS = 1
        const val WALLETS = 2
        const val PSBTS = 3
        const val NEW_KEY = 4
        const val PSBT = 5
        const val WALLET = 6
        const val KEY = 7
        const val IMPORT_PSBT = 8
        const val IMPORT_WALLET = 9
        const val DICE_FACES = 10
        const val DICE_LAUNCH = 11
        const val ADDRESS_INDEX = 12

        fun comeHere(from: Activity, what: Int) {
            val newIntent = Intent(from, ListActivity::class.java)
            newIntent.putExtra(C.WHAT, what)
            from.startActivityForResult(newIntent, what)
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
                title = "Keys"
                updateKeys()
                item_new.setOnClickListener {
                    comeHere(this, NEW_KEY)
                }
            }
            NEW_KEY -> {
                title = "New key"
                itemsAdapter.list.add(Item(getString( R.string.random), null, null, emptyList()))
                itemsAdapter.list.add(Item(getString( R.string.dice), null, null, emptyList()))
                itemsAdapter.list.add(Item(getString( R.string.import_xprv), null, null, emptyList()))
                itemsAdapter.list.add(Item(getString( R.string.import_mnemonic), null, null, emptyList()))
                item_new.hide()
            }
            WALLETS -> {
                title = "Wallets"
                updateWallets()
                item_new.setOnClickListener {
                    comeHere(this, IMPORT_WALLET)
                }
            }
            PSBTS -> {
                title = "Transactions"
                updatePsbts()
                item_new.setOnClickListener {
                    comeHere(this, IMPORT_PSBT)
                }
            }
            IMPORT_PSBT -> {
                title = "Import transaction (PSBT)"
                itemsAdapter.list.add(Item(getString(R.string.scan), "one or more qr codes", null, emptyList()))
                itemsAdapter.list.add(Item(getString(R.string.insert_manually), "base64", null, emptyList()))
                item_new.hide()
            }
            IMPORT_WALLET -> {
                title = "Import wallet"
                itemsAdapter.list.add(Item(getString(R.string.scan), "one or more qr codes", null, emptyList()))
                itemsAdapter.list.add(Item(getString(R.string.insert_manually), "json", null, emptyList()))
                item_new.hide()
            }
            DICE_FACES -> {
                title = "How many faces has the dice?"
                for (el in PERFECT_SOLID_FACES) {
                    itemsAdapter.list.add(Item(el.toString(), null, null, emptyList()))
                }
                item_new.hide()
            }
            DICE_LAUNCH -> {
                val a = intent.getIntExtra(C.LAUNCH_NUMBER,0)
                val b = launchesRequired(intent.getIntExtra(C.FACES,0))
                title = "$a dice launch of $b?"
                val faces = intent.getIntExtra(C.FACES, 0)
                for (el in (1..faces)) {
                    itemsAdapter.list.add(Item(el.toString(), null, null, emptyList()))
                }
                item_new.hide()
            }
            ADDRESS_INDEX -> {
                title = "Select index"
                for (i in 0..1000) {
                    itemsAdapter.list.add(Item("$i", null, null, emptyList()))
                }
                item_new.hide()
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
        integrator.setDesiredBarcodeFormats(IntentIntegrator.QR_CODE)
        integrator.setPrompt(title)
        integrator.initiateScan()
    }

    private fun updateKeys() {
        update("keys")
        for (key in listOutput.keys) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(key)
            itemsAdapter.list.add(Item(key.key.name, key.key.fingerprint, details, key.public_qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updateWallets() {
        update("wallets")
        for (wallet in listOutput.wallets) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(wallet)
            itemsAdapter.list.add(Item(wallet.wallet.name, wallet.wallet.fingerprints.toString(), details, wallet.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun updatePsbts() {
        update("psbts")
        for (psbt in listOutput.psbts) {
            val details = mapper.writerWithDefaultPrettyPrinter().writeValueAsString(psbt)
            itemsAdapter.list.add(Item(psbt.psbt.name, psbt.signatures, details, psbt.qr_files))
        }
        itemsAdapter.notifyDataSetChanged()
    }

    private fun update(kind: String) {
        try {
            itemsAdapter.list.clear()
            listOutput = Rust().list(filesDir.toString(), kind)
        } catch (e: RustException) {
            C.showMessageDialog(this, e.message?:"Null")
        }
    }

    override fun onItemLongClick(item: Item) {
        setResultAndFinish(item)
    }

    override fun onItemClick(item: Item) {
        val what = intent.getIntExtra(C.WHAT, 0)
        Log.d("LIST","onItemClick $item.name ${Network.TYPE} $what")

        when(what) {
            NEW_KEY -> {
                keyNameDialog(item.name)
            }
            KEYS -> {
                val newIntent = Intent(this, KeyActivity::class.java)
                newIntent.putExtra(C.KEY, item.json)
                startActivityForResult(newIntent, KEY)
            }
            WALLETS -> {
                val newIntent = Intent(this, WalletActivity::class.java)
                newIntent.putExtra(C.WALLET, item.json)
                startActivityForResult(newIntent, WALLET)
            }
            PSBTS -> {
                val newIntent = Intent(this, PSBTActivity::class.java)
                newIntent.putExtra(C.PSBT, item.json)
                startActivityForResult(newIntent, PSBT)
            }
            IMPORT_PSBT -> {
                when(item.name) {
                    getString(R.string.scan) -> {
                        launchScan("Scan a transaction (PSBT)")
                    }
                    getString(R.string.insert_manually) -> {
                        val valueEditText = EditText(this)
                        valueEditText.maxLines = 1
                        valueEditText.inputType = InputType.TYPE_CLASS_TEXT

                        val dialog: AlertDialog = AlertDialog.Builder(this)
                            .setTitle("Insert PSBT base64")
                            .setView(valueEditText)
                            .setPositiveButton("Ok") { _, _ ->
                                val text = valueEditText.text.toString()
                                savePsbt(text, "base64")
                                finish()
                            }
                            .setNegativeButton("Cancel", null)
                            .create()
                        dialog.show()
                    }
                }
            }
            IMPORT_WALLET -> {
                when(item.name) {
                    getString(R.string.scan) -> {
                        launchScan("Scan a Wallet")
                    }
                    getString(R.string.insert_manually) -> {
                        val valueEditText = EditText(this)
                        valueEditText.maxLines = 1
                        valueEditText.inputType = InputType.TYPE_CLASS_TEXT

                        val dialog: AlertDialog = AlertDialog.Builder(this)
                            .setTitle("Insert a JSON Wallet")
                            .setView(valueEditText)
                            .setPositiveButton("Ok") { _, _ ->
                                val text = valueEditText.text.toString()
                                saveWallet(text)
                                finish()
                            }
                            .setNegativeButton("Cancel", null)
                            .create()
                        dialog.show()
                    }
                }
            }
            DICE_FACES -> {
                setResultAndFinish(item)
            }
            DICE_LAUNCH -> {
                setResultAndFinish(item)
            }
            ADDRESS_INDEX -> {
                setResultAndFinish(item)
            }
            else -> {
                Log.w("LIST", "not mapped")
            }
        }
    }

    private fun setResultAndFinish(item: Item) {
        val returnIntent = Intent()
        returnIntent.putExtra(C.RESULT, item.name)
        setResult(Activity.RESULT_OK, returnIntent)
        finish()
    }

    private fun setResultMessage(id: Int) {
        setResultMessage(getString(id))
    }

    private fun setResultMessage(message: String) {
        val intent = Intent()
        intent.putExtra(C.SHOW_MESSAGE, message)
        setResult(Activity.RESULT_CANCELED, intent)
    }

    private fun valueDialog(name: String, nature: String) {
        val valueEditText = EditText(this)
        valueEditText.maxLines = 1
        valueEditText.inputType = InputType.TYPE_CLASS_TEXT

        val dialog: AlertDialog = AlertDialog.Builder(this)
            .setTitle("Insert $nature")
            .setView(valueEditText)
            .setPositiveButton("Ok") { _, _ ->
                val text = valueEditText.text.toString()
                try {
                    Rust().restore(filesDir.toString(), name, nature, text)
                    setResult(Activity.RESULT_OK, Intent())
                } catch (e: RustException) {
                    Log.e("LIST", e.message?:"Null")
                    setResultMessage(R.string.invalid_xprv_or_mnemonic)
                }
                finish()
            }
            .setNegativeButton("Cancel", null)
            .create()
        dialog.show()
    }

    private fun keyNameDialog(what: String) {
        val keyEditText = EditText(this)
        keyEditText.maxLines = 1
        keyEditText.inputType = InputType.TYPE_CLASS_TEXT

        val dialog: AlertDialog = AlertDialog.Builder(this)
            .setTitle("New key")
            .setMessage("Give this key a unique name.")
            .setView(keyEditText)
            .setPositiveButton("Ok") { _, _ ->
                val keyName = keyEditText.text.toString()
                if (keyName.isNotEmpty()) {
                    val keyFile = File("$filesDir/${Network.TYPE}/keys/$keyName/PRIVATE.json")
                    if (keyFile.exists()) {
                        C.showMessageDialog(this, R.string.key_exists)
                    } else {
                        when (what) {
                            getString(R.string.random) -> {
                                Rust().random(filesDir.toString(), keyName)
                                setResult(Activity.RESULT_OK, Intent())
                                finish()
                            }
                            getString(R.string.dice) -> {
                                this.keyName = keyName
                                comeHere(this, DICE_FACES)
                            }
                            getString(R.string.import_xprv) -> valueDialog(keyName, "Xprv")
                            getString(R.string.import_mnemonic) -> valueDialog(keyName, "Mnemonic")
                        }
                    }
                }
            }
            .setNegativeButton("Cancel", null)
            .create()
        dialog.show()
    }

    private fun saveWallet(content: String) {
        Log.d("LIST", "saveWallet $content")
        try {
            val json = mapper.readValue(content, Rust.WalletJson::class.java)
            Rust().importWallet(filesDir.toString(), json)
        } catch (e: Exception) {
            Log.e("LIST", e.message?:"Null")
            setResultMessage(R.string.wallet_not_imported)
        }
    }

    private fun savePsbt(psbt: String, encoding: String) {
        Log.d("LIST", "savePsbt ${psbt.length} chars length, encoding: $encoding")
        try {
            Rust().savePSBT(filesDir.toString(), psbt, encoding)
        } catch (e: Exception) {
            Log.e("LIST", e.message?:"Null")
            setResultMessage(R.string.not_a_psbt)
        }
    }

    override fun onResume() {
        super.onResume()
        when(intent.getIntExtra(C.WHAT, 0)) {
            PSBTS -> updatePsbts()
            WALLETS -> updateWallets()
            KEYS -> updateKeys()
        }
    }

    override fun onActivityResult(
        requestCode: Int,
        resultCode: Int,
        data: Intent?
    ) {
        val result = IntentIntegrator.parseActivityResult(requestCode, resultCode, data)
        if (result != null) {
            if (result.contents == null) {
                rawHexes.clear()
                C.showMessageDialog(this, R.string.cancelled)
            } else {
                val hexString = result.rawBytes.toHexString()
                this.rawHexes.add(hexString)
                if (hexString.startsWith("3")) {
                    try {
                        val hexResult = Rust().mergeQrs(filesDir.toString(), this.rawHexes)
                        rawHexes.clear()
                        Log.d("MAIN", "qr complete: $hexResult")
                        when (intent.getIntExtra(C.WHAT, 0)) {
                            IMPORT_WALLET -> {
                                val bytes = decodeHexString(hexResult)
                                saveWallet(bytes!!.toString(Charsets.UTF_8))
                                finish()
                            }
                            IMPORT_PSBT -> {
                                savePsbt(hexResult, "hex")
                                finish()
                            }
                        }
                    } catch (e: RustException) {
                        launchScan("Next")
                    }
                } else {
                    when (intent.getIntExtra(C.WHAT, 0)) {
                        WALLETS -> {
                            saveWallet(result.contents)
                            updateWallets()
                        }
                        PSBTS -> {
                            savePsbt(result.contents, "base64")
                            updatePsbts()
                        }
                    }
                }
            }
        } else if (requestCode == NEW_KEY && resultCode == Activity.RESULT_OK) {
            updateKeys()
        } else if (requestCode == DICE_FACES && resultCode== Activity.RESULT_OK) {
            faces = data?.getStringExtra(C.RESULT)!!.toInt()
            Log.d("LIST", "faces are $faces")
            launchDice(1)
        } else if (requestCode == DICE_LAUNCH && resultCode== Activity.RESULT_OK) {
            val launch = data?.getStringExtra(C.RESULT)!!
            Log.d("LIST", "launch is $launch")
            diceLaunches.add(launch.toInt())
            if (diceLaunches.size == launchesRequired(faces)) {
                Log.d("LIST", "finish diceLaunches $diceLaunches")
                Rust().dice(filesDir.toString(), keyName!!, faces.toString(), diceLaunches)
                resetFields()
                finish()
            } else {
                launchDice(diceLaunches.size+1)
            }
        } else if (requestCode in arrayOf(PSBT,WALLET,KEY) && resultCode == Activity.RESULT_OK) {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, data!!.getStringExtra(C.RESULT))
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        } else {
            Log.i("LIST", "result cancelled")
            data?.let { C.showMessageIfInIntent(this, it) }
            resetFields()
            super.onActivityResult(requestCode, resultCode, data)
        }
    }

    private fun resetFields() {
        diceLaunches.clear()
        faces = 0
        keyName = null
    }

    private fun launchesRequired(faces: Int) = LAUNCHES_FOR_256_BIT[PERFECT_SOLID_FACES.indexOf(faces)]

    private fun launchDice(launchNumber: Int) {
        val newIntent = Intent(this, ListActivity::class.java)
        newIntent.putExtra(C.WHAT, DICE_LAUNCH)
        newIntent.putExtra(C.FACES, faces)
        newIntent.putExtra(C.LAUNCH_NUMBER, launchNumber)
        startActivityForResult(newIntent, DICE_LAUNCH)
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

class ItemsAdapter : RecyclerView.Adapter<ItemHolder>() {

    val list: ArrayList<Item> = ArrayList()
    var listener: ItemGesture? = null

    override fun getItemCount():Int{
        return list.size
    }

    override fun onBindViewHolder(holder: ItemHolder, position: Int) {
        val tx = list[position]
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
        val item = LayoutInflater.from(parent.context).inflate(R.layout.key_item, parent, false)
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
