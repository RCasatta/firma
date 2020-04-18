package it.casatta

import android.app.Application
import android.util.Log
import it.casatta.ListActivity.Companion.network
import java.io.File


class FirmaApplication : Application() {
    companion object {
        init {
            System.loadLibrary("firma")
        }
    }

    override fun onCreate() {
        super.onCreate()

        if (BuildConfig.DEBUG) {
            Log.d("APP", "inserting debug data")
            val networkDir = File(filesDir, "testnet")
            val psbts = File(networkDir, "psbts")
            val psbt = File(psbts, "debugpsbt")
            if (!psbt.exists()) {
                psbt.mkdirs()
                val desc = File(psbt, "psbt.json")
                Log.d("APP", "creating ${desc}")
                desc.writeText("{\n" +
                        "  \"name\": \"debugpsbt\",\n" +
                        "  \"psbt\": \"cHNidP8BAIkCAAAAAVVd7uAtyLPar5aayKPxo+xFgs7KihUhAPQ30WM/lanqAQAAAAD+////AjOZBAAAAAAAIgAgJYnpLTM4WFxxDTx0HpR4GRG3z96GNLK9skbczk0rp0+5OPEFAAAAACIAIL2+BqQBABGX1fT3+YEf2JTiSEm9Z/KaoVOsi8/X8nZ1AAAAAAABASsA4fUFAAAAACIAICWJ6S0zOFhccQ08dB6UeBkRt8/ehjSyvbJG3M5NK6dPAQVHUiECaBLnyMLxU4GgHycMnQJEtnl5RRUyNCLjq4lFU4nuSBIhAv4fKttAP1uMccRwEAdIYW7c795LtmrS99xUk4IphGblUq4iBgJoEufIwvFTgaAfJwydAkS2eXlFFTI0IuOriUVTie5IEgxQZExdAAAAAAAAAAAiBgL+HyrbQD9bjHHEcBAHSGFu3O/eS7Zq0vfcVJOCKYRm5Qy1FFKIAAAAAAAAAAAAAQFHUiECaBLnyMLxU4GgHycMnQJEtnl5RRUyNCLjq4lFU4nuSBIhAv4fKttAP1uMccRwEAdIYW7c795LtmrS99xUk4IphGblUq4iAgJoEufIwvFTgaAfJwydAkS2eXlFFTI0IuOriUVTie5IEgxQZExdAAAAAAAAAAAiAgL+HyrbQD9bjHHEcBAHSGFu3O/eS7Zq0vfcVJOCKYRm5Qy1FFKIAAAAAAAAAAAAAQFHUiEDkkURPn0O5+lqCqGEOXKufEqfSR8OTkzUeDDMk2E+SAMhAm8XZKxRFGG8Uc9O65ADAam9nhBQruD4NS9LnRc++VwOUq4iAgJvF2SsURRhvFHPTuuQAwGpvZ4QUK7g+DUvS50XPvlcDgy1FFKIAQAAAAAAAAAiAgOSRRE+fQ7n6WoKoYQ5cq58Sp9JHw5OTNR4MMyTYT5IAwxQZExdAQAAAAAAAAAA\",\n" +
                        "  \"fee\": 0.0000386,\n" +
                        "  \"changepos\": 1\n" +
                        "}\n")
                Rust().create_qrs(desc.toString(), "testnet")
            }

            val wallets = File(networkDir, "wallets")
            val wallet = File(wallets, "debugwallet")
            if (!wallet.exists()) {
                wallet.mkdirs()
                val desc = File(wallet, "descriptor.json")
                Log.d("APP", "creating ${desc}")
                desc.writeText("{\n" +
                        "  \"name\": \"debugwallet\",\n" +
                        "  \"descriptor_main\": \"wsh(multi(2,tpubD6NzVbkrYhZ4Y9aAwHWSycLZwdMGaD1azej6XetqXXZ95ZRKzEr43guTG9hrVb7LMo1SkapzwDtMdrkZYFL7T5mn8UF9sEsPnSMvjRu8UcQ/0/*,tpubD6NzVbkrYhZ4XP4JMRUFsTvCnrdrKEFN3FcziJ8xZoACvBrK6X79zsUJTSH26qxymWyydvHwsoBHGFDnvo2ryq5vUztrdj3Mtkx5s6Texwm/0/*))#g26ame55\",\n" +
                        "  \"descriptor_change\": \"wsh(multi(2,tpubD6NzVbkrYhZ4Y9aAwHWSycLZwdMGaD1azej6XetqXXZ95ZRKzEr43guTG9hrVb7LMo1SkapzwDtMdrkZYFL7T5mn8UF9sEsPnSMvjRu8UcQ/1/*,tpubD6NzVbkrYhZ4XP4JMRUFsTvCnrdrKEFN3FcziJ8xZoACvBrK6X79zsUJTSH26qxymWyydvHwsoBHGFDnvo2ryq5vUztrdj3Mtkx5s6Texwm/1/*))#t22g7aq8\",\n" +
                        "  \"fingerprints\": [\n" +
                        "    \"50644c5d\",\n" +
                        "    \"b5145288\"\n" +
                        "  ],\n" +
                        "  \"required_sig\": 2,\n" +
                        "  \"created_at_height\": 101\n" +
                        "}\n"
                )
                Rust().create_qrs(desc.toString(), "testnet")
            }

        }
    }
}