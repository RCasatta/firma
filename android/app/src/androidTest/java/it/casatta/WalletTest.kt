package it.casatta

import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
import androidx.test.espresso.assertion.ViewAssertions.matches
import androidx.test.espresso.matcher.ViewMatchers.*
import androidx.test.ext.junit.rules.activityScenarioRule
import androidx.test.ext.junit.runners.AndroidJUnit4
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
    val activityScenarioRule = activityScenarioRule<MainActivity>()

    @Test
    fun wallet() {

        val descriptorMainMainnet =
            "wsh(multi(2,xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk/0/*,xpub661MyMwAqRbcFL1pGULVsWqCN3tRyneHcTKq8rzvt6Mh9vwG5sPM2QPU4pFdRkqi9SMu7S35CNve2gjxPLtHhQVKhMuUoEtfPnjePzX2xWk/0/*))#q0agyfvx";
        val descriptorMainTestnet =
            "wsh(multi(2,tpubD6NzVbkrYhZ4XeQW5Adf6Cho9eaBWTzoCApPf2NGsyFCYx2WVEFWQ9hmuwdJi3WbnG33CqAqFGrZYVrZeUztHoUGmPaxqzp96w2oMu9JCUV/0/*,tpubD6NzVbkrYhZ4WrwU2gJn1bJ1UrZ4kPnGAwXY384rpDhHJmcs2xJkmLm17dF1zpvC1roPWVXqiy2U4Up5dQp94ep1hjjQYS5vUArfT5kP92y/0/*))#q0agyfvx";
        val mainDescriptors = mapOf(
            "mainnet" to descriptorMainMainnet,
            "testnet" to descriptorMainTestnet,
            "regtest" to descriptorMainTestnet
        )
        val network = getNetwork()

        val name = "wallet-${System.currentTimeMillis()}"
        val wallet = Data.WalletJson(
            name,
            mainDescriptors[network]!!,
            listOf("8f335370", "6b9128bc"),
            2,
            1718227,
            null
        )
        val walletString = mapper.writeValueAsString(wallet)

        onView(withId(R.id.wallet_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(getString(R.string.insert_manually))
        setTextInDialogAndConfirm(walletString)
        onView(withText(name)).check(matches(isDisplayed()))
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(getString(R.string.insert_manually))
        setTextInDialogAndConfirm( walletString)
        checkAndDismissDialog(R.string.wallet_not_imported)
        clickElementInList(name)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(name, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.wallet_button)).perform(click())
        checkElementNotInList(name)

        val invalidNetwork = invalidNetwork(network)
        val invalidWallet = Data.WalletJson(
            name,
            mainDescriptors[invalidNetwork]!!,
            listOf("8f335370", "6b9128bc"),
            2,
            1718227,
            null
        )
        val invalidWalletString = mapper.writeValueAsString(invalidWallet)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(getString(R.string.insert_manually))
        setTextInDialogAndConfirm(invalidWalletString)
        checkAndDismissDialog(R.string.wallet_not_imported)

    }

}
