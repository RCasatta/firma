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
import kotlin.random.Random


/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class ExampleInstrumentedTest {
    @Test
    fun useAppContext() {
        // Context of the app under test.
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertTrue(appContext.packageName.startsWith("it.casatta"))
    }

    @get:Rule
    var activityRule: ActivityTestRule<MainActivity> = ActivityTestRule(
        MainActivity::class.java,
        true,  // initialTouchMode
        false
    ) // launchActivity. False to customize the intent

    @Test
    fun createNewRandomKey() {
        activityRule.launchActivity(Intent())
        val randomKeyName = randomKeyName()

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
        val randomKeyName = randomKeyName()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        onView(withId(R.id.items_list)).perform(RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(withItemSubject("Import tprv"), click()))
        onView(withClassName(containsString("EditText"))).inRoot(isDialog()).perform(typeText(randomKeyName))
        onView(withText("OK")).perform(click())
        onView(withClassName(containsString("EditText"))).inRoot(isDialog()).perform(typeText("tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF"))
        onView(withText("OK")).perform(click())
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
        onView(withId(R.id.items_list)).perform(RecyclerViewActions.actionOnHolderItem<RecyclerView.ViewHolder>(withItemSubject(randomKeyName), click()))
        onView(withText("tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj")).check(matches(isDisplayed()))

    }

    private val charPool : List<Char> = ('a'..'z') + ('A'..'Z') + ('0'..'9')
    private fun randomKeyName(): String {
        return (1..12)
            .map { i -> Random.nextInt(0, charPool.size) }
            .map(charPool::get)
            .joinToString("")
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
