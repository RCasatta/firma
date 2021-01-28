package it.casatta

import android.app.Activity
import android.content.Intent
import android.graphics.BitmapFactory
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import androidx.recyclerview.widget.DividerItemDecoration
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.json.Data
import java.io.Serializable


class QrActivity : ContextActivity() {
    private val imagesAdapter = ImageItemsAdapter()
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    companion object {
        fun comeHere(from: Activity, titlePrefix: String, qrContent: Data.StringEncoding) {
            val newIntent = Intent(from, QrActivity::class.java)
            newIntent.putExtra(C.QR_CONTENT, Data.decodeStringEncoding(qrContent))
            newIntent.putExtra(C.TITLE_PREFIX, titlePrefix)
            from.startActivityForResult(newIntent, 0)
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_qr)

        val recyclerView = findViewById<RecyclerView>(R.id.qrs_list)

        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.addItemDecoration(
            DividerItemDecoration(
                recyclerView.context,
                DividerItemDecoration.VERTICAL
            )
        )
        recyclerView.adapter = imagesAdapter

        val qrContent = Data.encodeStringEncodingHex( intent.getByteArrayExtra(C.QR_CONTENT)!! )
        val encodedQrs = Rust().qrs(context(), qrContent)

        for (qrBmp in encodedQrs.qrs) {
            imagesAdapter.list.add(ImageItem( Data.decodeStringEncoding(qrBmp)))
        }

        val titlePrefix = intent.getStringExtra(C.TITLE_PREFIX)
        title = if (imagesAdapter.list.size<=1) {
            "$titlePrefix - QR code"
        } else {
            "$titlePrefix - ${imagesAdapter.list.size} QR codes"
        }
    }
}

data class ImageItem(val qrContent: ByteArray): Serializable

class ImageItemsAdapter : RecyclerView.Adapter<ImageItemHolder>(){

    val list: ArrayList<ImageItem> = ArrayList()

    override fun getItemCount():Int{
        return list.size
    }

    override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ImageItemHolder {
        val item = LayoutInflater.from(parent.context).inflate(R.layout.image_item, parent, false)
        return ImageItemHolder(item)
    }
    override fun onBindViewHolder(holder: ImageItemHolder, position: Int) {
        val image = list[position]
        holder.update(image)
    }
}

class ImageItemHolder(itemView: View): RecyclerView.ViewHolder(itemView) {
    private val imageView = itemView.findViewById<ImageView>(R.id.qr_image)

    fun update(item: ImageItem) {
        val decodedImage = BitmapFactory.decodeByteArray(item.qrContent, 0, item.qrContent.size)
        imageView.setImageBitmap(decodedImage)
    }
}

