package it.casatta

import androidx.test.espresso.Espresso
import androidx.test.espresso.Espresso.onView
import androidx.test.espresso.action.ViewActions.click
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
class SignTest : Common() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    @get:Rule
    val activityScenarioRule = activityScenarioRule<MainActivity>()

    @Test
    fun sign() {
        val network = getNetwork()
        if ("mainnet" != network) {
            val aliceTprv = "tprv8ZgxMBicQKsPfEf2t9eG7j14CDjS3JWL9nY3wgg6ZsLKY4tsR4wZjYuLsXWdyBPrMPo73JgeKmbd8pTkZZgQNWTdvCtDuauf52XGKL9zTDw"
            val aliceTpub = "tpubD6NzVbkrYhZ4YhgpmoJrX8fAmFFNCdhEj68qECiPz98iNZ9e3Tm9v3XD3fzHZfBoLqeSm9oLtighoeijQ9jGAFm9raQ4JqHZ1N4BHyaBz6Y"
            val aliceKeyName = "alice_sign_test"
            val bobTprv = "tprv8ZgxMBicQKsPetwSbvkSob1PLvNeBzHftBgG61S37ywMpsCnKMkUhPbKp7FyZDsU2QvMqbF797DRqmwedPQnR5qqmUBkFVb7iNeKcEZv3ck"
            val bobTpub = "tpubD6NzVbkrYhZ4YMyEVaR3CzfVuwtaMKUaTVH3NXULYFjkfMTYwka4stDBzHhHkxd4MEMVgyyEV1WBCrpwde72w8LzjAE6oRLARBAiCD8cGQV"
            val bobKeyName = "bob_sign_test"
            val required_sig = 2;
            val walletName = "alice-and-bob"
            val descriptor = "wsh(multi($required_sig,$aliceTpub/0/*,$bobTpub/0/*))"
            val wallet = Data.WalletJson(
                walletName,
                descriptor,
                listOf("1f5e43d8", "a2ebe04e"),
                required_sig,
                1835680,
                null
            )
            val walletString = mapper.writeValueAsString(wallet)

            val tx = "cHNidP8BAFMCAAAAASFSbAAqstjwTxbGtWir21+meBp5LMcUQsBSgZ5bDtD7AQAAAAD+////AV6rCAAAAAAAF6kU4wEfjwloN3dvCV9wNOekdO53E92HAAAAAAX8bmFtZQh0by1jYXJvbAABAKECAAAAAcyd+J9zW1wSNV/mozPMv8mcXFzwQrK1EKq/FvRPJS40AQAAACMiACC+U25ZjJg9CiGsPhlAqQ0GWtFhOWxqopXdDTrh2oBdEP3///8Cp0lVAAAAAAAXqRRUIuqRoByuLh5D6zdViHWG7aGi84cVrAgAAAAAACIAIDz80EGjAUinXjMddGAtfQ3fKqcjgWj9wY5Y+8c7NA1zoAIcAAEBKxWsCAAAAAAAIgAgPPzQQaMBSKdeMx10YC19Dd8qpyOBaP3Bjlj7xzs0DXMBBUdSIQNP26ruccaqcu2cxRFYsPON2gj4ALrAFQ5ApBVtM+z9SiECIwjICs3MMHNnGbXPgSQKezAcOC5HzejKyjATzR8qXiRSriIGAiMIyArNzDBzZxm1z4EkCnswHDguR83oysowE80fKl4kDB9eQ9gAAAAAAAAAACIGA0/bqu5xxqpy7ZzFEViw843aCPgAusAVDkCkFW0z7P1KDKLr4E4AAAAAAAAAAAAA"
            val txName = "to-carol"

            /// START importing key, wallet and tx
            onView(withId(R.id.key_button)).perform(click())

            onView(withId(R.id.item_new)).perform(click())
            clickElementInList("Import tprv")
            setTextInDialogAndConfirm(aliceKeyName)
            setTextInDialogAndConfirm(aliceTprv)

            onView(withId(R.id.item_new)).perform(click())
            clickElementInList("Import tprv")
            setTextInDialogAndConfirm(bobKeyName)
            setTextInDialogAndConfirm(bobTprv)

            Espresso.pressBack()

            onView(withId(R.id.wallet_button)).perform(click())
            onView(withId(R.id.item_new)).perform(click())
            clickElementInList(getString(R.string.insert_manually))
            setTextInDialogAndConfirm(walletString)

            Espresso.pressBack()

            onView(withId(R.id.psbt_button)).perform(click())
            onView(withId(R.id.item_new)).perform(click())
            clickElementInList(getString(R.string.insert_manually))
            setTextInDialogAndConfirm(tx)

            Espresso.pressBack()
            /// END importing key, wallet and tx

            /// START selecting key, wallet and tx
            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(aliceKeyName)
            onView(withId(R.id.select)).perform(click())

            onView(withId(R.id.wallet_button)).perform(click())
            clickElementInList(walletName)
            onView(withId(R.id.select)).perform(click())

            onView(withId(R.id.psbt_button)).perform(click())
            clickElementInList(txName)
            onView(withId(R.id.select)).perform(click())
            /// END selecting key, wallet and tx

            /// START signing
            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog(R.string.added_signatures)

            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog("request to sign a PSBT already containing a signature from this key")

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(bobKeyName)
            onView(withId(R.id.select)).perform(click())

            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog(R.string.added_signatures)

            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog("request to sign a PSBT already containing a signature from this key")
            /// END signing

            /// START deleting key, wallet and tx
            onView(withId(R.id.psbt_button)).perform(click())
            clickElementInList(txName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(txName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.wallet_button)).perform(click())
            clickElementInList(walletName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(walletName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(aliceKeyName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(aliceKeyName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(bobKeyName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(bobKeyName, "DELETE")
            checkAndDismissDialog(R.string.deleted)
            /// END deleting key, wallet and tx
        }
    }


}
