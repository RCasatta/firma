package it.casatta

import android.content.Context
import android.content.Intent
import android.widget.EditText
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import java.io.File

class C {
    companion object {
        const val WHAT = "what"
        const val RESULT = "result"
        const val QRS = "qrs"
        const val PSBT = "psbt"
        const val WALLET = "wallet"
        const val INDEX = "index"
        const val DESCRIPTOR = "descriptor"
        const val KEY = "key"
        const val TITLE_PREFIX = "title_prefix"
        const val FACES = "faces"
        const val LAUNCH_NUMBER = "launch_number"

        fun showDeleteDialog(context: Context, name: String, dir: String) {
            val valueEditText = EditText(context)
            AlertDialog.Builder(context)
                .setTitle("Type \"$name\" to delete")
                .setView(valueEditText)
                .setPositiveButton("Delete") { _, _ ->
                    if (valueEditText.text.toString() == name ) {
                        File(dir).deleteRecursively()
                        val intent = Intent(context, MainActivity::class.java)
                        intent.flags =
                            Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK
                        context.startActivity(intent)
                        Toast.makeText(context, "Deleted", Toast.LENGTH_LONG).show()
                    }
                }
                .setNegativeButton("Cancel", null)
                .create().show()
        }
    }


}