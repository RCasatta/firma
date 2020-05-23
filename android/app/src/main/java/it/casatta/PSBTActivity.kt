package it.casatta

import android.app.Activity
import android.content.Intent
import android.opengl.Visibility
import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import kotlinx.android.synthetic.main.activity_psbt.*
import java.io.File
import java.io.Serializable
import java.util.*
import kotlin.collections.ArrayList


class PSBTActivity : AppCompatActivity() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())
    private val inputsAdapter = TxInOutAdapter()
    private val outputsAdapter = TxInOutAdapter()
    private val itemsAdapter = DescItemAdapter()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_psbt)

        inputs.layoutManager = LinearLayoutManager(this)
        inputs.adapter = inputsAdapter

        outputs.layoutManager = LinearLayoutManager(this)
        outputs.adapter = outputsAdapter

        val psbtString = intent.getStringExtra(C.PSBT)
        Log.d("PSBT", "${Network.TYPE} $psbtString")
        val psbtJson = mapper.readValue(psbtString, Rust.PsbtJsonOutput::class.java)
        val psbtFileDir = "$filesDir/${Network.TYPE}/psbts/${psbtJson.psbt.name}/"
        val psbtFileName = "$psbtFileDir/psbt.json"
        val psbtPretty = Rust().print(filesDir.toString(), psbtFileName)

        val psbtTitle = "Transaction: ${psbtJson.psbt.name}"
        title = psbtTitle
        view_qr.setOnClickListener { QrActivity.comeHere(this, psbtTitle, psbtJson.qr_files ) }
        select.setOnClickListener {
            val returnIntent = Intent()
            returnIntent.putExtra(C.RESULT, psbtJson.psbt.name)
            setResult(Activity.RESULT_OK, returnIntent)
            finish()
        }

        delete.setOnClickListener {
            C.showDeleteDialog(this, psbtJson.psbt.name, psbtFileDir)
        }

        for (i in psbtPretty.inputs.indices) {
            val input = psbtPretty.inputs[i]
            var description: String? = null
            if (input.wallet_with_path != null && input.signatures.isNotEmpty() ) {
                description = "${input.wallet_with_path} ${input.signatures.joinToString(", ")}"
            } else if (input.wallet_with_path != null) {
                description = input.wallet_with_path
            } else if (input.signatures.isNotEmpty()) {
                description = input.signatures.joinToString(", ")
            }

            inputsAdapter.list.add(TxInOutItem("input #$i", input.outpoint, input.value, description))
        }

        for (i in psbtPretty.outputs.indices) {
            val output = psbtPretty.outputs[i]
            outputsAdapter.list.add(TxInOutItem("output #$i", output.address, output.value, output.wallet_with_path ))
        }

        items.layoutManager = LinearLayoutManager(this)
        items.adapter = itemsAdapter

        val info = psbtPretty.info.joinToString()
        if (info.isNotEmpty()) {
            itemsAdapter.list.add(DescItem("Info", info))
        }
        val formattedRate = String.format(Locale.US, "%.2f sat/vB", psbtPretty.fee.rate)
        itemsAdapter.list.add(DescItem("Fee", psbtPretty.fee.absolute_fmt))
        itemsAdapter.list.add(DescItem("Fee rate", formattedRate))
        itemsAdapter.list.add(DescItem("Balances", psbtPretty.balances))
        itemsAdapter.list.add(DescItem("Estimated size", "${psbtPretty.size.estimated} bytes"))
        itemsAdapter.list.add(DescItem("Unsigned size", "${psbtPretty.size.unsigned} bytes"))
        itemsAdapter.list.add(DescItem("PSBT size", "${psbtPretty.size.psbt} bytes"))
        itemsAdapter.list.add(DescItem("PSBT", psbtJson.psbt.psbt))

    }
}

data class TxInOutItem(val index: String, val title: String, val value: String, val description: String?): Serializable

class TxInOutAdapter : RecyclerView.Adapter<TxInOutItemHolder>(){

    val list: ArrayList<TxInOutItem> = ArrayList()

    override fun getItemCount():Int{
        return list.size
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): TxInOutItemHolder {
        val item = LayoutInflater.from(parent.context).inflate(R.layout.txinout_item, parent, false)
        return TxInOutItemHolder(item)
    }
    override fun onBindViewHolder(holder: TxInOutItemHolder, position: Int) {
        val item = list[position]
        holder.update(item)
    }
}

class TxInOutItemHolder(itemView: View): RecyclerView.ViewHolder(itemView) {
    private val index = itemView.findViewById<TextView>(R.id.index)
    private val title = itemView.findViewById<TextView>(R.id.title)
    private val value = itemView.findViewById<TextView>(R.id.value)
    private val description = itemView.findViewById<TextView>(R.id.description)

    fun update(item: TxInOutItem) {
        index.text = item.index
        title.text = item.title
        value.text = item.value
        if (item.description == null) {
            description.visibility = View.GONE
        } else {
            description.text = item.description
        }
    }
}




