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
    fun randomKey() {
        val activity = activityRule.launchActivity(Intent())
        val randomKeyName = "key${System.currentTimeMillis()}"

        onView(withId(R.id.key_button)).perform(click())

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        onView(withText(randomKeyName)).check(matches(isDisplayed()))

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        checkAndDismissDialog(R.string.key_exists)

        onView(isRoot()).perform(pressBack())
        clickElementInList(randomKeyName)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialog(activity, randomKeyName)
        onView(withText("DELETE")).perform(click())
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(randomKeyName)
    }

    @Test
    fun xprv() {
        val activity = activityRule.launchActivity(Intent())
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
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        setTextInDialog(activity, xprvs[network]!!)
        clickDialogOK()
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
        clickElementInList(randomKeyName)
        onView(withText(xpubs[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialog(activity, randomKeyName)
        onView(withText("DELETE")).perform(click())
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(randomKeyName)

        val invalidNetwork = invalidNetwork(network)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(importsText[network]!!)
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        setTextInDialog(activity, xprvs[invalidNetwork]!!)
        clickDialogOK()
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
    }

    @Test
    fun mnemonic() {
        val activity = activityRule.launchActivity(Intent())
        val randomKeyName = "key${System.currentTimeMillis()}"

        val expectedXpubTestnet =
            "tpubD6NzVbkrYhZ4WUShmaCWa9ZQwAVe2kKxyfY1sENpyNfaQhjLHuS82RLjz19gaFTRknZhmSVAbzbeE79RjTb5coEjsjA4yg9seCLK8EFm5Q6"
        val expectedXpubMainnet =
            "xpub661MyMwAqRbcEgFrKPDRNEE3c3Pz3MpuS2wiWgKgdTugfR4dDgb2syfujy4u3kWBxt2o9ryJ4tmbXvepLbjqjMnSp8QmJ9Ve4EjuNBkPMy1"
        val expectedXpub = mapOf(
            "mainnet" to expectedXpubMainnet,
            "testnet" to expectedXpubTestnet,
            "regtest" to expectedXpubTestnet
        )
        val mnemonic =
            "bunker shed useless about build taste comfort acquire food defense nation cement oblige race manual narrow merit lumber slight pattern plate budget armed undo"
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.import_mnemonic))
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        setTextInDialog(activity, "invalid")
        clickDialogOK()
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.import_mnemonic))
        setTextInDialog(activity, randomKeyName)
        clickDialogOK()
        setTextInDialog(activity, mnemonic)
        clickDialogOK()
        onView(withText(randomKeyName)).check(matches(isDisplayed()))
        clickElementInList(randomKeyName)
        onView(withText(expectedXpub[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialog(activity, randomKeyName)
        onView(withText("DELETE")).perform(click())
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(randomKeyName)
    }

    @Test
    fun dice() {

    }
}
