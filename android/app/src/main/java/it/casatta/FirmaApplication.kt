package it.casatta

import android.app.Application
import android.util.Log
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
            val psbt = File(psbts, "psbt-0")
            if (!psbt.exists()) {
                val hex="70736274ff0100890200000001555deee02dc8b3daaf969ac8a3f1a3ec4582ceca8a152100f437d1633f95a9ea0100000000feffffff0233990400000000002200202589e92d3338585c710d3c741e94781911b7cfde8634b2bdb246dcce4d2ba74fb938f10500000000220020bdbe06a401001197d5f4f7f9811fd894e24849bd67f29aa153ac8bcfd7f27675000000000001012b00e1f505000000002200202589e92d3338585c710d3c741e94781911b7cfde8634b2bdb246dcce4d2ba74f0105475221026812e7c8c2f15381a01f270c9d0244b679794515323422e3ab89455389ee48122102fe1f2adb403f5b8c71c470100748616edcefde4bb66ad2f7dc549382298466e552ae2206026812e7c8c2f15381a01f270c9d0244b679794515323422e3ab89455389ee48120c50644c5d0000000000000000220602fe1f2adb403f5b8c71c470100748616edcefde4bb66ad2f7dc549382298466e50cb51452880000000000000000000101475221026812e7c8c2f15381a01f270c9d0244b679794515323422e3ab89455389ee48122102fe1f2adb403f5b8c71c470100748616edcefde4bb66ad2f7dc549382298466e552ae2202026812e7c8c2f15381a01f270c9d0244b679794515323422e3ab89455389ee48120c50644c5d0000000000000000220202fe1f2adb403f5b8c71c470100748616edcefde4bb66ad2f7dc549382298466e50cb51452880000000000000000000101475221039245113e7d0ee7e96a0aa1843972ae7c4a9f491f0e4e4cd47830cc93613e480321026f1764ac511461bc51cf4eeb900301a9bd9e1050aee0f8352f4b9d173ef95c0e52ae2202026f1764ac511461bc51cf4eeb900301a9bd9e1050aee0f8352f4b9d173ef95c0e0cb514528801000000000000002202039245113e7d0ee7e96a0aa1843972ae7c4a9f491f0e4e4cd47830cc93613e48030c50644c5d010000000000000000";
                Rust().savePSBT(filesDir.toString(),hex)
            }

            val wallets = File(networkDir, "wallets")
            val wallet = File(wallets, "debugwallet")
            if (!wallet.exists()) {
                wallet.mkdirs()
                val desc = File(wallet, "descriptor.json")
                Log.d("APP", "creating $desc")
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
                Rust().createQrs(desc.toString())
            }

        }
    }
}