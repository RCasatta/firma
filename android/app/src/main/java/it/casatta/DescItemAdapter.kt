package it.casatta

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import java.io.Serializable

data class DescItem(val title: String, val value: String): Serializable
class DescItemAdapter() : RecyclerView.Adapter<DescItemHolder>(){

    val list: ArrayList<DescItem> = ArrayList()

    override fun getItemCount():Int{
        return list.size
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): DescItemHolder {
        var item = LayoutInflater.from(parent?.context)
            .inflate(R.layout.desc_item, parent, false)
        return DescItemHolder(item)
    }
    override fun onBindViewHolder(holder: DescItemHolder, position: Int) {
        var item = list[position]
        holder?.update(item)
    }
}

class DescItemHolder(itemView: View): RecyclerView.ViewHolder(itemView) {
    private val title = itemView.findViewById<TextView>(
        R.id.title
    )
    private val value = itemView.findViewById<TextView>(
        R.id.value
    )

    fun update(item: DescItem) {

        title.text = item.title
        value.text = item.value

    }
}