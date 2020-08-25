package it.casatta

import android.view.View
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import androidx.test.espresso.Espresso
import androidx.test.espresso.action.ViewActions
import androidx.test.espresso.assertion.ViewAssertions
import androidx.test.espresso.contrib.RecyclerViewActions
import androidx.test.espresso.matcher.BoundedMatcher
import androidx.test.espresso.matcher.RootMatchers
import androidx.test.espresso.matcher.ViewMatchers
import androidx.test.internal.util.Checks
import androidx.test.platform.app.InstrumentationRegistry
import org.hamcrest.Description
import org.hamcrest.Matcher
import org.hamcrest.Matchers
import org.junit.Assert

open class Common {

    fun invalidNetwork(network: String): String {
        val validNetworks = arrayOf("mainnet", "testnet", "regtest")
        Assert.assertTrue(validNetworks.contains(network))
        when (network) {
            "mainnet" -> return "testnet"
            "testnet" -> return "mainnet"
            "regtest" -> return "mainnet"
        }
        return ""
    }

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

    fun checkAndDismissDialog(id: Int) {
        Espresso.onView(ViewMatchers.withText(id))
            .inRoot(RootMatchers.isDialog()).check(ViewAssertions.matches(ViewMatchers.isDisplayed()))
            .perform(ViewActions.pressBack())
    }

    fun clickElementInList(subject: String) {
        Espresso.onView(ViewMatchers.withId(R.id.items_list)).perform(
            RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(
                withItemSubject(subject),
                ViewActions.click()
            )
        )
    }

    fun clickDialogOK() {
        Espresso.onView(ViewMatchers.withText("OK")).perform(ViewActions.click())
    }

    fun typeInDialog(randomKeyName: String) {
        Espresso.onView(ViewMatchers.withClassName(Matchers.containsString("EditText")))
            .inRoot(RootMatchers.isDialog())
            .perform(ViewActions.typeText(randomKeyName))
    }
}