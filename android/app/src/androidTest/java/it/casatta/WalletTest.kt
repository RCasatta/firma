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
            "wsh(multi(1,[dd0847bb/48'/0'/0'/2']xpub6En6P3aEhpmH9DqU9QpiMEL94QWDsNTVnVW8gqi6W2TBU7z4kPDenHLrNkzihcYhEvkRehZfC67uF1Sn8oqq9Q7nxnHPPEL96vawmCQZgVp/0/*))#7kvyp5du";
        val descriptorMainTestnet =
            "wsh(multi(1,[d90c6a4f/48'/1'/0'/2']tpubDFk5MPbkQ9zKfgmmLkS9buF12Enr2JiWyDfwucm7oxwM5Y3uDWrzEJ4Q8VQbQwXoFTz9A7QTTHDr8soGzYoJoWKtfxn8vfHtquFv8poghnf/0/*))#02gvp2va";
        val mainDescriptors = mapOf(
            "mainnet" to descriptorMainMainnet,
            "testnet" to descriptorMainTestnet,
            "regtest" to descriptorMainTestnet
        )

        val name = "wallet-${System.currentTimeMillis()}"
        val identifier = Data.Identifier(Data.Kind.WALLET, name, Network.TYPE)
        val wallet = Data.WalletJson(
            identifier,
            mainDescriptors[network]!!,
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
        val identifier2 = Data.Identifier(Data.Kind.WALLET, name, Network.TYPE)
        val invalidWallet = Data.WalletJson(
            identifier2,
            mainDescriptors[invalidNetwork]!!,
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
