package it.casatta

import android.app.Activity
import android.content.Intent
import android.graphics.BitmapFactory
import android.os.Bundle
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ImageView
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.DividerItemDecoration
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import java.io.File
import java.io.Serializable


class QrActivity : AppCompatActivity() {
    private val imagesAdapter = ImageItemsAdapter()

    companion object {
        fun comeHere(from: Activity, titlePrefix: String, files: List<String>) {
            val newIntent = Intent(from, QrActivity::class.java)
            newIntent.putStringArrayListExtra(C.QRS, ArrayList(files))
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

        val qrs = intent.getStringArrayListExtra(C.QRS)
        Log.d("QR", qrs.toString())

        var count = 0
        for (qr in qrs) {
            if (qr.endsWith(".png")) {
                imagesAdapter.list.add(ImageItem(qr))
                count += 1
            }
        }

        val titlePrefix = intent.getStringExtra(C.TITLE_PREFIX)
        title = if (count<=1) {
            "$titlePrefix - QR code"
        } else {
            "$titlePrefix - $count QR codes"
        }
    }
}

data class ImageItem(val name: String): Serializable

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
        val imgFile = File(item.name)

        if (imgFile.exists()) {
            val myBitmap = BitmapFactory.decodeFile(imgFile.absolutePath)
            imageView.setImageBitmap(myBitmap)
        }
    }
}

