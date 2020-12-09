package it.casatta

import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.action.ViewActions.pressBack
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.*
import androidx.test.ext.junit.rules.activityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class KeyTest : Common() {
    @get:Rule
    val activityScenarioRule = activityScenarioRule<MainActivity>()

    @Test
    fun randomKey() {
        val keyName = "key${System.currentTimeMillis()}"

        onView(withId(R.id.key_button)).perform(click())

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialogAndConfirm(keyName)
        onView(withText(keyName)).check(matches(isDisplayed()))

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialogAndConfirm(keyName)
        checkAndDismissDialog(R.string.key_exists)

        onView(isRoot()).perform(pressBack())
        clickElementInList(keyName)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)
    }

    @Test
    fun xprv() {
        val keyName = "key${System.currentTimeMillis()}"
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
        setTextInDialogAndConfirm(keyName)
        setTextInDialogAndConfirm(xprvs[network]!!)
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)
        onView(withText(xpubs[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)

        val invalidNetwork = invalidNetwork(network)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(importsText[network]!!)
        setTextInDialogAndConfirm(keyName)
        setTextInDialogAndConfirm(xprvs[invalidNetwork]!!)
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
    }

    @Test
    fun mnemonic() {
        val keyName = "key${System.currentTimeMillis()}"

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
        clickElementInList(getString(R.string.import_mnemonic))
        setTextInDialogAndConfirm(keyName)
        setTextInDialogAndConfirm("invalid")
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(getString(R.string.import_mnemonic))
        setTextInDialogAndConfirm(keyName)
        setTextInDialogAndConfirm(mnemonic)
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)
        onView(withText(expectedXpub[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)
    }

    @Test
    fun dice() {
        val keyName = "key${System.currentTimeMillis()}"
        val expectedXpubTestnet =
            "tpubD6NzVbkrYhZ4YSC7guz8W7xZW1ftPPwsB9bAcEHrmdvmzyUSfhTDE8YV3M8WYegAmGorTpGvVGVKdXS5gWkCQ7GPZNUABkchvCyNpA51h5b"
        val expectedXpubMainnet =
            "xpub661MyMwAqRbcGe1GEj13JCdCAtaEQ1SodWzsFgEiRjAtFgojbUc85gseoK3j29ivyNGwrEm3xAfGwLwUHetxWfp6VmirWDxULFNy4CD94UP"
        val expectedXpub = mapOf(
            "mainnet" to expectedXpubMainnet,
            "testnet" to expectedXpubTestnet,
            "regtest" to expectedXpubTestnet
        )
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(getString(R.string.dice))
        setTextInDialogAndConfirm(keyName)
        clickElementInList("20")
        for (i in 1..59) {
            clickElementInList("2")
        }
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)

        onView(withText(expectedXpub[network]!!)).check(matches(isDisplayed()))
    }
}
