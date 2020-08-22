package it.casatta

import android.content.Intent
import android.view.View
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.action.ViewActions.typeText
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.contrib.RecyclerViewActions
import androidx.test.espresso.matcher.BoundedMatcher
import androidx.test.espresso.matcher.RootMatchers.isDialog
import androidx.test.espresso.matcher.ViewMatchers.*
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.internal.util.Checks
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.rule.ActivityTestRule
import org.hamcrest.Description
import org.hamcrest.Matcher
import org.hamcrest.Matchers.containsString
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class InstrumentedTest {

    private fun getNetwork(): String {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertTrue(appContext.packageName.startsWith("it.casatta"))
        val network = appContext.packageName.substring(11)
        val validNetworks = arrayOf("mainnet", "testnet", "regtest")
        assertTrue(validNetworks.contains(network))
        return network
    }

    @get:Rule
    var activityRule: ActivityTestRule<MainActivity> = ActivityTestRule(
        MainActivity::class.java,
        true,
        false
    )

    @Test
    fun createNewRandomKey() {
        activityRule.launchActivity(Intent())
        val randomKeyName = "key${ System.currentTimeMillis()}"

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        onView(withId(R.id.items_list)).perform(RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(withItemSubject("Random"), click()))
        onView(withClassName(containsString("EditText"))).inRoot(isDialog()).perform(typeText(randomKeyName))
        onView(withText("OK")).perform(click())
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
    }

    @Test
    fun importXprv() {
        activityRule.launchActivity(Intent())
        val randomKeyName = "key${ System.currentTimeMillis()}"
        val xprvs = mapOf("mainnet" to "xprv9s21ZrQH143K2qwMASoVWNtTp23waKvSFEQELUbKKkpiH8c7YL56Uc4zDWrTgyeUrMsDxEt7CuGg3PZBwdygrMa3b4KTSowCQ7LEv48AaRQ", "testnet" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF", "regtest" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF")
        val xpubs = mapOf("mainnet" to "xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk", "testnet" to "tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj", "regtest" to "tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj")
        val importsText = mapOf("mainnet" to "Import xprv", "testnet" to "Import tprv", "regtest" to "Import tprv")
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        onView(withId(R.id.items_list)).perform(RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(withItemSubject(importsText[network]!!), click()))
        onView(withClassName(containsString("EditText"))).inRoot(isDialog()).perform(typeText(randomKeyName))
        onView(withText("OK")).perform(click())
        onView(withClassName(containsString("EditText"))).inRoot(isDialog()).perform(typeText(xprvs[network]))
        onView(withText("OK")).perform(click())
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
        onView(withId(R.id.items_list)).perform(RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(withItemSubject(randomKeyName), click()))
        onView(withText(xpubs[network])).check(matches(isDisplayed()))
    }

    private fun withItemSubject(subject: String): Matcher<RecyclerView.ViewHolder?>? {
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
