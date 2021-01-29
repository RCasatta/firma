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

    private fun callMethod(context: Context, method: String, args: Any): JsonNode {
        val req = JsonRpc(method, context, args)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun list(context: Context, kind: Kind): ListOutput {
        val opt = ListOptions(kind, true)
        val json = callMethod(context, "list",  opt)
        return mapper.convertValue(json, ListOutput::class.java)
    }

    fun random(context: Context, keyName: String): JsonNode {
        val opt = RandomOptions(keyName)
        return callMethod(context, "random", opt)
    }

    fun dice(context: Context, keyName: String, faces: Base, launches: ArrayList<Int>): JsonNode {
        val opt = DiceOptions(keyName,  faces, Bits._256, launches)
        return callMethod(context,"dice", opt)
    }

    fun mergeQrs(context: Context, qrs_content: List<StringEncoding>): StringEncoding {
        val opt = QrMergeOptions(qrs_content)
        val json =  callMethod(context,"merge_qrs",  opt)
        return mapper.convertValue(json, StringEncoding::class.java)
    }

    fun qrs(context: Context, qr_content: StringEncoding): EncodedQrs {
        val opt = QrOptions(qr_content, 14)
        val json =  callMethod(context,"qrs",  opt)
        return mapper.convertValue(json, EncodedQrs::class.java)
    }

    fun importWallet(context: Context, wallet: WalletJson) {
        callMethod(context,"import", wallet)
    }

    fun signWallet(context: Context, walletName: String) {
        val opt = WalletNameOptions(walletName)
        callMethod(context, "sign_wallet", opt)
    }

    fun verifyWallet(context: Context, walletName: String): VerifyWalletResult {
        val opt = WalletNameOptions(walletName)
        val json = callMethod(context, "verify_wallet", opt)
        return mapper.convertValue(json, VerifyWalletResult::class.java)
    }

    fun sign(context: Context, key: String, wallet: String, psbt: String): PsbtPrettyPrint {
        val opt = SignOptions(key, wallet, psbt, 100, false )
        val json = callMethod(context, "sign", opt)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun restore(context: Context, key: String, nature: Nature, value: String): JsonNode {
        val opt = RestoreOptions(key, nature, value)
        return callMethod(context,"restore", opt)
    }

    fun print(context: Context, psbt_file: String): PsbtPrettyPrint {
        val opt = PrintOptions(psbt_file, null , null, true)
        val json = callMethod(context,"print", opt)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun savePSBT(context: Context, psbt: StringEncoding) {
        val opt = SavePSBTOptions(psbt)
        callMethod(context,"save_psbt", opt)
    }

    fun deriveAddress(context: Context, walletDescriptor: String, addressIndex: Int): GetAddressOutput {
        val opt = DeriveAddressOpts(walletDescriptor, addressIndex)
        val json = callMethod(context, "derive_address", opt)
        return mapper.convertValue(json, GetAddressOutput::class.java)
    }
}

class RustException(message: String) : Exception(message)

