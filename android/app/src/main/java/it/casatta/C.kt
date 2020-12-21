package it.casatta

import android.content.Context
import android.content.Intent
import android.widget.EditText
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
        const val SHOW_MESSAGE = "show_message"

        fun showDeleteDialog(context: Context, name: String, dir: String) {
            val valueEditText = EditText(context)
            AlertDialog.Builder(context)
                .setTitle("Type \"$name\" to delete")
                .setView(valueEditText)
                .setPositiveButton("Delete") { _, _ ->
                    if (valueEditText.text.toString() == name) {
                        File(dir).deleteRecursively()
                        val intent = Intent(context, MainActivity::class.java)
                        intent.flags =
                            Intent.FLAG_ACTIVITY_CLEAR_TASK or Intent.FLAG_ACTIVITY_NEW_TASK
                        intent.putExtra(SHOW_MESSAGE, context.getString(R.string.deleted))
                        context.startActivity(intent)
                    }
                }
                .setNegativeButton("Cancel", null)
                .create().show()
        }

        /**
         * Use showMessageDialog directly only if you are not calling finish soon after, in that case
         * use setResultMessage and let the destination activity show the dialog
         */
        fun showMessageDialog(context: Context, id: Int) {
            showMessageDialog(context, context.getString(id))
        }
        fun showMessageDialog(context: Context, message: String) {
            AlertDialog.Builder(context)
                .setMessage(message)
                .setCancelable(true)
                .create().show()
        }
        fun showMessageIfInIntent(context: Context, intent: Intent) {
            val message = intent.getStringExtra(SHOW_MESSAGE)
            if (message != null && message.isNotEmpty()) {
                showMessageDialog(context, message)
            }
        }

    }


}