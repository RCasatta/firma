package it.casatta

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule
import it.casatta.json.Data.*
import it.casatta.json.Options.*

class Rust {
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    private external fun call(json: String): String

    private fun callJson(json: String): JsonNode {
        val strResult = call(json)
        val jsonResult = mapper.readTree(strResult)
        if (jsonResult.has("error")) {
            throw RustException(jsonResult["error"].asText())
        }
        return jsonResult
    }

    private fun callMethod(method: String, datadir: String, network: String, args: Any): JsonNode {
        val req = JsonRpc(method, datadir, network, args)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun list(datadir: String, kind: Kind, encryptionKey: StringEncoding): ListOutput {
        val encryptionKeys = listOf(encryptionKey)
        val opt = ListOptions(kind, encryptionKeys)
        val json = callMethod("list", datadir, Network.TYPE, opt)
        return mapper.convertValue(json, ListOutput::class.java)
    }

    fun random(datadir: String, keyName: String, encryptionKey: StringEncoding): JsonNode {
        val opt = RandomOptions(keyName, 14, encryptionKey)
        return callMethod("random", datadir, Network.TYPE, opt)
    }

    fun dice(datadir: String, keyName: String, faces: Base, launches: ArrayList<Int>, encryptionKey: StringEncoding): JsonNode {
        val opt = DiceOptions(keyName, 14, encryptionKey, faces, Bits._256, launches)
        return callMethod("dice", datadir, Network.TYPE, opt)
    }

    fun mergeQrs(datadir: String, qrs_bytes: List<String>): String {
        return callMethod("merge_qrs", datadir, Network.TYPE, qrs_bytes).asText()
    }

    fun importWallet(datadir: String, wallet: WalletJson) {
        callMethod("import_wallet", datadir, Network.TYPE, wallet)
    }

    fun sign(datadir: String, key: String, wallet: String, psbt: String, encryptionKey: StringEncoding): PsbtPrettyPrint {
        val opt = SignOptions(key, 100, wallet, 14, psbt, false, encryptionKey )
        val json = callMethod("sign", datadir, Network.TYPE, opt)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun restore(datadir: String, key: String, nature: Nature, value: String, encryptionKey: StringEncoding): JsonNode {
        val opt = RestoreOptions(key, nature, 14, encryptionKey, value)
        return callMethod("restore", datadir, Network.TYPE, opt)
    }

    fun print(datadir: String, psbt_file: String): PsbtPrettyPrint {
        val opt = PrintOptions(psbt_file)
        val json = callMethod("print", datadir, Network.TYPE, opt)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun savePSBT(datadir: String, psbt: StringEncoding) {
        val opt = SavePSBTOptions(psbt, 14)
        callMethod("save_psbt", datadir, Network.TYPE, opt)
    }

    fun deriveAddress(walletDescriptor: String, addressIndex: Int): GetAddressOutput {
        val opt = DeriveAddressOpts(walletDescriptor, addressIndex)
        val json = callMethod("derive_address", "", Network.TYPE, opt)
        return mapper.convertValue(json, GetAddressOutput::class.java)
    }
}

class RustException(message: String) : Exception(message)

