package it.casatta

import androidx.appcompat.app.AppCompatActivity
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.KotlinModule

import it.casatta.json.Data
import it.casatta.json.Options

open class ContextActivity : AppCompatActivity() {
    private val mapper = ObjectMapper().registerModule(KotlinModule())
    private val context: Data.Context by lazy {
        Data.Context(filesDir.toString(), Network.TYPE, EncryptionKey.get(applicationContext))
    }

    private fun callJson(json: String): JsonNode {
        val strResult = Rust().call(json)
        val jsonResult = mapper.readTree(strResult)
        if (jsonResult.has("error")) {
            throw RustException(jsonResult["error"].asText())
        }
        return jsonResult
    }

    private fun callMethod(method: String, args: Any): JsonNode {
        val req = Data.JsonRpc(method, context, args)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun list(kind: Data.Kind): Data.ListOutput {
        val opt = Options.ListOptions(kind)
        val json = callMethod( "list",  opt)
        return mapper.convertValue(json, Data.ListOutput::class.java)
    }

    fun random(keyName: String): JsonNode {
        val opt = Options.RandomOptions(keyName)
        return callMethod( "random", opt)
    }

    fun dice(keyName: String, faces: Data.Base, launches: ArrayList<Int>): JsonNode {
        val opt = Options.DiceOptions(keyName, faces, Data.Bits._256, launches)
        return callMethod("dice", opt)
    }

    fun mergeQrs(qrs_content: List<Data.StringEncoding>): Data.StringEncoding {
        val opt = Options.QrMergeOptions(qrs_content)
        val json =  callMethod("merge_qrs",  opt)
        return mapper.convertValue(json, Data.StringEncoding::class.java)
    }

    fun qrs(qr_content: Data.StringEncoding): Data.EncodedQrs {
        val opt = Options.QrOptions(qr_content, 14)
        val json =  callMethod("qrs",  opt)
        return mapper.convertValue(json, Data.EncodedQrs::class.java)
    }

    fun importWallet(wallet: Data.WalletJson) {
        callMethod("import", wallet)
    }

    fun exportSignature(name: String): Data.WalletSignature {
        val opt = Options.ExportOptions(Data.Kind.WALLET_SIGNATURE, name)
        val json = callMethod("export", opt)
        return mapper.convertValue(json, Data.WalletSignature::class.java)
    }

    fun signWallet(walletName: String) {
        val opt = Options.WalletNameOptions(walletName)
        callMethod( "sign_wallet", opt)
    }

    fun verifyWallet(walletName: String): Options.VerifyWalletResult {
        val opt = Options.WalletNameOptions(walletName)
        val json = callMethod( "verify_wallet", opt)
        return mapper.convertValue(json, Options.VerifyWalletResult::class.java)
    }

    fun sign(key: String, wallet: String, psbt: String): Data.PsbtPrettyPrint {
        val opt = Options.SignOptions(key, wallet, psbt, 100, false)
        val json = callMethod( "sign", opt)
        return mapper.convertValue(json, Data.PsbtPrettyPrint::class.java)
    }

    fun restore(key: String, nature: Data.Nature, value: String): JsonNode {
        val opt = Options.RestoreOptions(key, nature, value)
        return callMethod("restore", opt)
    }

    fun print(psbt_file: String): Data.PsbtPrettyPrint {
        val opt = Options.PrintOptions(psbt_file, null, null, true)
        val json = callMethod("print", opt)
        return mapper.convertValue(json, Data.PsbtPrettyPrint::class.java)
    }

    fun savePSBT(psbt: Data.StringEncoding) {
        val opt = Options.SavePSBTOptions(psbt)
        callMethod("save_psbt", opt)
    }

    fun deriveAddress(walletDescriptor: String, addressIndex: Int): Data.GetAddressOutput {
        val opt = Options.DeriveAddressOpts(walletDescriptor, addressIndex)
        val json = callMethod("derive_address", opt)
        return mapper.convertValue(json, Data.GetAddressOutput::class.java)
    }




}

class RustException(message: String) : Exception(message)