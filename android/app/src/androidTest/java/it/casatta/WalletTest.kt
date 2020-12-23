package it.casatta

import android.content.Intent
import androidx.test.espresso.Espresso
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.*
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.rule.ActivityTestRule
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.json.Data
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith


@RunWith(AndroidJUnit4::class)
class WalletTest : Common() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    @get:Rule
    var activityRule: ActivityTestRule<MainActivity> = ActivityTestRule(
        MainActivity::class.java,
        true,
        false
    )

    @Test
    fun wallet() {
        val activity = activityRule.launchActivity(Intent())

        // first import a key, otherwise the wallet will not be imported
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
        setTextInDialogAndConfirm(activity, keyName)
        setTextInDialogAndConfirm(activity, xprvs[network]!!)
        onView(withText(keyName)).check(matches(isDisplayed()))
        Espresso.pressBack()

        // import the wallet
        val descriptorMainMainnet =
            "wsh(multi(2,xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk/0/*,xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk/0/*))#q0agyfvx";
        val descriptorMainTestnet =
            "wsh(multi(2,tpubD6NzVbkrYhZ4WcVS4H3kcnSZYJfNbofW2oF3AWFXSK86vwRwiB2EqfF9vUyxVC9ZxDkVGZo9xvSLYxfVsBWdcQHKbN9xbE7iPp9eRgbgpfj/0/*,tpubD6NzVbkrYhZ4WrwU2gJn1bJ1UrZ4kPnGAwXY384rpDhHJmcs2xJkmLm17dF1zpvC1roPWVXqiy2U4Up5dQp94ep1hjjQYS5vUArfT5kP92y/0/*))#q0agyfvx";
        val mainDescriptors = mapOf(
            "mainnet" to descriptorMainMainnet,
            "testnet" to descriptorMainTestnet,
            "regtest" to descriptorMainTestnet
        )

        val name = "wallet-${System.currentTimeMillis()}"
        val wallet = Data.WalletJson(
            name,
            mainDescriptors[network]!!,
            listOf("8f335370", "6b9128bc"),
            2,
            1718227
        )
        val walletString = mapper.writeValueAsString(wallet)

        onView(withId(R.id.wallet_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.insert_manually))
        setTextInDialogAndConfirm(activity, walletString)
        onView(withText(name)).check(matches(isDisplayed()))
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.insert_manually))
        setTextInDialogAndConfirm(activity, walletString)
        checkAndDismissDialog(R.string.wallet_not_imported)
        clickElementInList(name)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(activity, name, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.wallet_button)).perform(click())
        checkElementNotInList(name)

        val invalidNetwork = invalidNetwork(network)
        val invalidWallet = Data.WalletJson(
            name,
            mainDescriptors[invalidNetwork]!!,
            listOf("8f335370", "6b9128bc"),
            2,
            1718227
        )
        val invalidWalletString = mapper.writeValueAsString(invalidWallet)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.insert_manually))
        setTextInDialogAndConfirm(activity, invalidWalletString)
        checkAndDismissDialog(R.string.wallet_not_imported)
        Espresso.pressBack()

        //select key
        onView(withId(R.id.key_button)).perform(click())
        clickElementInList(keyName)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(activity, keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)
    }

}
