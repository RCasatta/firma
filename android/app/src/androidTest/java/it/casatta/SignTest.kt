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
class SignTest : Common() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    @get:Rule
    var activityRule: ActivityTestRule<MainActivity> = ActivityTestRule(
        MainActivity::class.java,
        true,
        false
    )

    @Test
    fun sign() {
        val activity = activityRule.launchActivity(Intent())
        val network = getNetwork()
        if ("mainnet" != network) {
            val aliceTprv = "tprv8ZgxMBicQKsPfEf2t9eG7j14CDjS3JWL9nY3wgg6ZsLKY4tsR4wZjYuLsXWdyBPrMPo73JgeKmbd8pTkZZgQNWTdvCtDuauf52XGKL9zTDw"
            val aliceKeyName = "alice_sign_test"
            val bobTprv = "tprv8ZgxMBicQKsPetwSbvkSob1PLvNeBzHftBgG61S37ywMpsCnKMkUhPbKp7FyZDsU2QvMqbF797DRqmwedPQnR5qqmUBkFVb7iNeKcEZv3ck"
            val bobKeyName = "bob_sign_test"
            val walletName = "alice-and-bob"
            //val descriptor = "wsh(multi(2,tpubD6NzVbkrYhZ4YhgpmoJrX8fAmFFNCdhEj68qECiPz98iNZ9e3Tm9v3XD3fzHZfBoLqeSm9oLtighoeijQ9jGAFm9raQ4JqHZ1N4BHyaBz6Y/0/*,tpubD6NzVbkrYhZ4YMyEVaR3CzfVuwtaMKUaTVH3NXULYFjkfMTYwka4stDBzHhHkxd4MEMVgyyEV1WBCrpwde72w8LzjAE6oRLARBAiCD8cGQV/0/*))#wss3kl0z"
            val descriptor = "wsh(multi(2,[a2ebe04e/48'/1'/0'/2']tpubDEXDRpvW2srXCSjAvC36zYkSE3jxT1wf7JXDo35Ln4NZpmaMNhq8o9coH9U9BQ5bAN4WDGxXV9d426iYKGorFF5wvv4Wv63cZsCotiXGGkD/0/*,[1f5e43d8/48'/1'/0'/2']tpubDFU4parcXvV8tBYt4rS4a8rGNF1DA32DCnRfhzVL6b3MSiDomV95rv9mb7W7jAPMTohyEYpbhVS8FbmTsuQsFRxDWPJX2ZFEeRPMFz3R1gh/0/*))#szg2xsau"
            val identifier = Data.Identifier(Data.Kind.WALLET, walletName, Network.TYPE)

            val wallet = Data.WalletJson(
                identifier,
                descriptor,
                1835680
            )
            val walletString = mapper.writeValueAsString(wallet)

            val tx = "cHNidP8BAH4CAAAAAQQYGYyRDjWA/D08BEjU3Q9P34Sv8q0mW9UV5niEqBZ4AQAAAAD+////AiDLAAAAAAAAF6kUaV+OwCj7iV87pOHOFXNLuZMc7tyHBwIAAAAAAAAiACAGYNwSo/z0dYfDuCUPL2Li/SSY10gjxu8hZ9pREpEaCwAAAAAM/AVmaXJtYQBuYW1lCHRvLWNhcm9sAAEAoQIAAAABG7mL63lJDPOLQybsXY8WZhK8QMjvz5D/qM6KBtZAYmQAAAAAIyIAIPynXT2ph1cCtzZ2E+fD0d6vmuZPc8BQvMyVxOjcK+c1/f///wJMiwYAAAAAABepFGdxKLPj9gk9IONcwMW/kz2S7YYIh6TOAAAAAAAAIgAg9ZFXIhxr0C/u7qGjb+y5bdnmVPnY3tH583t2S8HyPqp+hR0AAQErpM4AAAAAAAAiACD1kVciHGvQL+7uoaNv7Llt2eZU+dje0fnze3ZLwfI+qgEFR1IhApKznFtt8+fKlGOcjgKzwmEgy8O2et7atlNfdA5bb80uIQN9dFnXvgcdA4fmLWblwKJbuzazugS3dzc6PrlDq2fd4FKuIgYCkrOcW23z58qUY5yOArPCYSDLw7Z63tq2U190DltvzS4couvgTjAAAIABAACAAAAAgAIAAIAAAAAAAAAAACIGA310Wde+Bx0Dh+YtZuXAolu7NrO6BLd3Nzo+uUOrZ93gHB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAAAAAAAAAEBR1IhAuOCnowHNpvquGET8SUCHqm7lSymqSslu2U4B2VdZ9hAIQOo4hJeqVo5DnlJPz/2YUn3odyLWIHI1GBOEbzdokJRf1KuIgIC44KejAc2m+q4YRPxJQIeqbuVLKapKyW7ZTgHZV1n2EAcouvgTjAAAIABAACAAAAAgAIAAIAAAAAAAQAAACICA6jiEl6pWjkOeUk/P/ZhSfeh3ItYgcjUYE4RvN2iQlF/HB9eQ9gwAACAAQAAgAAAAIACAACAAAAAAAEAAAAA"
            val txName = "to-carol"

            /// START importing key, wallet and tx
            onView(withId(R.id.key_button)).perform(click())

            onView(withId(R.id.item_new)).perform(click())
            clickElementInList("Import tprv")
            setTextInDialogAndConfirm(activity, aliceKeyName)
            setTextInDialogAndConfirm(activity, aliceTprv)

            onView(withId(R.id.item_new)).perform(click())
            clickElementInList("Import tprv")
            setTextInDialogAndConfirm(activity, bobKeyName)
            setTextInDialogAndConfirm(activity, bobTprv)

            Espresso.pressBack()

            onView(withId(R.id.wallet_button)).perform(click())
            onView(withId(R.id.item_new)).perform(click())
            clickElementInList(activity.getString(R.string.insert_manually))
            setTextInDialogAndConfirm(activity, walletString)

            Espresso.pressBack()

            onView(withId(R.id.psbt_button)).perform(click())
            onView(withId(R.id.item_new)).perform(click())
            clickElementInList(activity.getString(R.string.insert_manually))
            setTextInDialogAndConfirm(activity, tx)

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
            checkAndDismissDialog("This transaction already contains a signature from this key matching the one generated by us (RFC6979 complaint)")

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(bobKeyName)
            onView(withId(R.id.select)).perform(click())

            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog(R.string.added_signatures)

            onView(withId(R.id.sign_button)).perform(click())
            checkAndDismissDialog("This transaction already contains a signature from this key matching the one generated by us (RFC6979 complaint)")
            /// END signing

            /// START deleting key, wallet and tx
            onView(withId(R.id.psbt_button)).perform(click())
            clickElementInList(txName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(activity, txName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.wallet_button)).perform(click())
            clickElementInList(walletName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(activity, walletName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(aliceKeyName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(activity, aliceKeyName, "DELETE")
            checkAndDismissDialog(R.string.deleted)

            onView(withId(R.id.key_button)).perform(click())
            clickElementInList(bobKeyName)
            onView(withId(R.id.delete)).perform(click())
            setTextInDialogAndConfirm(activity, bobKeyName, "DELETE")
            checkAndDismissDialog(R.string.deleted)
            /// END deleting key, wallet and tx
        }
    }


}
