package it.casatta

import android.content.Intent
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.action.ViewActions.pressBack
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.*
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.rule.ActivityTestRule
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class KeyTest : Common() {

    @get:Rule
    var activityRule: ActivityTestRule<MainActivity> = ActivityTestRule(
        MainActivity::class.java,
        true,
        false
    )

    @Test
    fun createDeleteRandomKey() {
        activityRule.launchActivity(Intent())
        val randomKeyName = "key${System.currentTimeMillis()}"

        onView(withId(R.id.key_button)).perform(click())

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        typeInDialog(randomKeyName)
        clickDialogOK()
        onView(withText(randomKeyName)).check(matches(isDisplayed()))

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        typeInDialog(randomKeyName)
        clickDialogOK()
        checkAndDismissDialog(R.string.key_exists)

        onView(isRoot()).perform(pressBack())
        clickElementInList(randomKeyName)
        onView(withId(R.id.delete)).perform(click())
        typeInDialog(randomKeyName)
        onView(withText("DELETE")).perform(click())
        checkAndDismissDialog(R.string.deleted)
    }

    @Test
    fun importXprv() {
        activityRule.launchActivity(Intent())
        val randomKeyName = "key${System.currentTimeMillis()}"
        val xprvs = mapOf(
            "mainnet" to "xprv9s21ZrQH143K2qwMASoVWNtTp23waKvSFEQELUbKKkpiH8c7YL56Uc4zDWrTgyeUrMsDxEt7CuGg3PZBwdygrMa3b4KTSowCQ7LEv48AaRQ",
            "testnet" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF",
            "regtest" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF"
        )
        val xpubs = mapOf(
            "mainnet" to "xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk",
            "testnet" to "tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj",
            "regtest" to "tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj"
        )
        val importsText = mapOf(
            "mainnet" to "Import xprv",
            "testnet" to "Import tprv",
            "regtest" to "Import tprv"
        )
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(importsText[network]!!)
        typeInDialog(randomKeyName)
        clickDialogOK()
        typeInDialog(xprvs[network]!!)
        clickDialogOK()
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
        clickElementInList(randomKeyName)
        onView(withText(xpubs[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        typeInDialog(randomKeyName)
        onView(withText("DELETE")).perform(click())
        checkAndDismissDialog(R.string.deleted)
    }

}
