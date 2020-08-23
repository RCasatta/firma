package it.casatta

import android.view.View
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import androidx.test.espresso.matcher.BoundedMatcher
import androidx.test.internal.util.Checks
import androidx.test.platform.app.InstrumentationRegistry
import org.hamcrest.Description
import org.hamcrest.Matcher
import org.junit.Assert

class Common {
    companion object {
        fun getNetwork(): String {
            val appContext = InstrumentationRegistry.getInstrumentation().targetContext
            Assert.assertTrue(appContext.packageName.startsWith("it.casatta"))
            val network = appContext.packageName.substring(11)
            val validNetworks = arrayOf("mainnet", "testnet", "regtest")
            Assert.assertTrue(validNetworks.contains(network))
            return network
        }

        fun withItemSubject(subject: String): Matcher<RecyclerView.ViewHolder?>? {
            Checks.checkNotNull(subject)
            return object : BoundedMatcher<RecyclerView.ViewHolder?, ItemHolder>(
                ItemHolder::class.java
            ) {
                override fun matchesSafely(viewHolder: ItemHolder): Boolean {
                    val subjectTextView =
                        viewHolder.itemView.findViewById(R.id.name) as TextView
                    return subject == subjectTextView.text
                        .toString() && subjectTextView.visibility == View.VISIBLE
                }

                override fun describeTo(description: Description) {
                    description.appendText("item with subject: $subject")
                }
            }
        }
    }
}