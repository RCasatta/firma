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
        val keyName = "key${System.currentTimeMillis()}"

        onView(withId(R.id.key_button)).perform(click())

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialogAndConfirm(activity, keyName)
        onView(withText(keyName)).check(matches(isDisplayed()))

        onView(withId(R.id.item_new)).perform(click())
        clickElementInList("Random")
        setTextInDialogAndConfirm(activity, keyName)
        checkAndDismissDialog(R.string.key_exists)

        onView(isRoot()).perform(pressBack())
        clickElementInList(keyName)
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(activity, keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)
    }

    @Test
    fun xprv() {
        val activity = activityRule.launchActivity(Intent())
        val keyName = "key${System.currentTimeMillis()}"
        val xprvs = mapOf(
            "mainnet" to "xprv9s21ZrQH143K2qwMASoVWNtTp23waKvSFEQELUbKKkpiH8c7YL56Uc4zDWrTgyeUrMsDxEt7CuGg3PZBwdygrMa3b4KTSowCQ7LEv48AaRQ",
            "testnet" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF",
            "regtest" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF",
            "signet" to "tprv8ZgxMBicQKsPd9TeAdPADNnSyH9SSUUbTVeFszDE23Ki6TBB5nCefAdHkK8Fm3qMQR6sHwA56zqRmKmxnHk37JkiFzvncDqoKmPWubu7hDF"
        )
        val xpubs = mapOf(
            "mainnet" to "[dd0847bb/48'/0'/0'/2']xpub6En6P3aEhpmH9DqU9QpiMEL94QWDsNTVnVW8gqi6W2TBU7z4kPDenHLrNkzihcYhEvkRehZfC67uF1Sn8oqq9Q7nxnHPPEL96vawmCQZgVp/0/*",
            "testnet" to "[d90c6a4f/48'/1'/0'/2']tpubDFk5MPbkQ9zKfgmmLkS9buF12Enr2JiWyDfwucm7oxwM5Y3uDWrzEJ4Q8VQbQwXoFTz9A7QTTHDr8soGzYoJoWKtfxn8vfHtquFv8poghnf/0/*",
            "regtest" to "[d90c6a4f/48'/2'/0'/2']tpubDFeFLK8vQySAay9YBtJYXezAd1gv9SQKz4LaivdHymkuomrZ7erMVWHk6aigz1u9NwdK5qKWmn8UKra7B4MY1HF78XRVYcU7TNvRxLeXnEc/0/*",
            "signet" to "[d90c6a4f/48'/3'/0'/2']tpubDFFWnN94Aa2jNHkFFNFoDRvzR5xHwv7koLxoWtEMAxCcsyceiMyW2vVEZodutVMd8cnKZMk5aiBzygExQQSwPe7moeu2qx63jFrh177ZYBD/0/*"
        )
        val importsText = mapOf(
            "mainnet" to "Import xprv",
            "testnet" to "Import tprv",
            "regtest" to "Import tprv",
            "signet" to "Import tprv"
        )
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(importsText[network]!!)
        setTextInDialogAndConfirm(activity, keyName)
        setTextInDialogAndConfirm(activity, xprvs[network]!!)
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)
        onView(withText(xpubs[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(activity, keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)

        val invalidNetwork = invalidNetwork(network)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(importsText[network]!!)
        setTextInDialogAndConfirm(activity, keyName)
        setTextInDialogAndConfirm(activity, xprvs[invalidNetwork]!!)
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
    }

    @Test
    fun mnemonic() {
        val activity = activityRule.launchActivity(Intent())
        val keyName = "key${System.currentTimeMillis()}"

        val expectedDescPubMainnet =
            "[cabe32d7/48'/0'/0'/2']xpub6DhVvf4GfRxVQZxcGTYFxwWPoL7GbXMe7nfP5Uhi5ZWbqvWdJsdJnwKkkcWiWbse2fBZn3RPiSjzpJwqNe8Zwqvv9DSjPqkkdUiegP97SVC/0/*"
        val expectedDescPubTestnet =
            "[cabe32d7/48'/1'/0'/2']tpubDERURuyFUBH1qfB38hVJFXRrG4fJ6SQS3jwixLAtvRAierc8pbmLF3wBWpiqeV4kXkCN2QvndGeo5wcWtNXCNymvSmBnWT9NgNcb2nbEWQv/0/*"
        val expectedDescPubSignet =
            "[cabe32d7/48'/3'/0'/2']tpubDEmpCZSfan5n7uG2HDe9CAh25JxgkGyvXGgXfPxQhaZ8wzqJ9bpJ6ASQkkwPyfHbTy7Ew4MWCzydicwfHSYC5hES6eNz1nC9ETLc92oBy2o/0/*"
        val expectedDescPubRegtest =
            "[cabe32d7/48'/2'/0'/2']tpubDEANPi9ucM4k8g549b4f1NUjtisSJz9SaKPxfmvrj3nGJCd9HLn3bpmJu5PLEwLV4rDQqazjwN2rHacjH2W2wV93S9WABHXHJVgHuAAyVHr/0/*"

        val expectedXpub = mapOf(
            "mainnet" to expectedDescPubMainnet,
            "testnet" to expectedDescPubTestnet,
            "signet" to expectedDescPubSignet,
            "regtest" to expectedDescPubRegtest
        )
        val mnemonic =
            "bunker shed useless about build taste comfort acquire food defense nation cement oblige race manual narrow merit lumber slight pattern plate budget armed undo"
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.import_mnemonic))
        setTextInDialogAndConfirm(activity, keyName)
        setTextInDialogAndConfirm(activity, "invalid")
        checkAndDismissDialog(R.string.invalid_xprv_or_mnemonic)
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.import_mnemonic))
        setTextInDialogAndConfirm(activity, keyName)
        setTextInDialogAndConfirm(activity, mnemonic)
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)
        onView(withText(expectedXpub[network])).check(matches(isDisplayed()))
        onView(withId(R.id.delete)).perform(click())
        setTextInDialogAndConfirm(activity, keyName, "DELETE")
        checkAndDismissDialog(R.string.deleted)
        onView(withId(R.id.key_button)).perform(click())
        checkElementNotInList(keyName)
    }

    @Test
    fun dice() {
        val activity = activityRule.launchActivity(Intent())
        val keyName = "key${System.currentTimeMillis()}"
        val expectedDescPubMainnet =
            "[9cf794b6/48'/0'/0'/2']xpub6EcBbnjoWQyB2sm1nDpiqymb629pCQmprDp22QWU8NXp81YHzcm98duacrav2s1bw3kCryM6UTknRDJcxHCxkHX3fcXrmsKy6QRWpDDpTqS/0/*"
        val expectedDescPubTestnet =
            "[9cf794b6/48'/1'/0'/2']tpubDF2XEjMTg94eFNAsX5jEK5nxA4Vs6VVjJQXgztKmSpGy5jiBNBgrmpqxcjTEzdnvfVG5U7SMLndPHJph9EHVMZie6HFYCt6XnXfFpwASqyq/0/*"
        val expectedDescPubSignet =
            "[9cf794b6/48'/3'/0'/2']tpubDEhd7RmezfBKFpBSxMxhXYKqsW4E2dBVT7Jmm9N5YWkEHiNJ6izuzbMyZdYwhDk2x6GeCkcZEV7x86kGf9YawrKZGZ6v2c9txMJ1vjdz4J1/0/*"
        val expectedDescPubRegtest =
            "[9cf794b6/48'/2'/0'/2']tpubDFKHxokA8JTGedRPmkFvCJ8CB2zqcHt3zuP7BwNarwZEkrhRnGpPzC9KkdtGj9KvYdf3hBU8N3CMa43yWMiLuB5W3f95TncHgSZtTaH5TTN/0/*"
        val expectedXpub = mapOf(
            "mainnet" to expectedDescPubMainnet,
            "testnet" to expectedDescPubTestnet,
            "signet" to expectedDescPubSignet,
            "regtest" to expectedDescPubRegtest
        )
        val network = getNetwork()

        onView(withId(R.id.key_button)).perform(click())
        onView(withId(R.id.item_new)).perform(click())
        clickElementInList(activity.getString(R.string.dice))
        setTextInDialogAndConfirm(activity, keyName)
        clickElementInList("20")
        for (i in 1..59) {
            clickElementInList("2")
        }
        onView(withText(keyName)).check(matches(isDisplayed()))
        clickElementInList(keyName)

        onView(withText(expectedXpub[network]!!)).check(matches(isDisplayed()))
    }
}
