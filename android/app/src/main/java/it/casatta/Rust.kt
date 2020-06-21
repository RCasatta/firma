package it.casatta

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.JsonNodeFactory
import com.fasterxml.jackson.module.kotlin.KotlinModule

class Rust {
    private val mapper = ObjectMapper().registerModule(KotlinModule())

    data class JsonRpc(
        val method: String,
        val datadir: String,
        val network: String,
        val args: JsonNode
    )

    data class ListOutput(
        val keys: List<MasterKeyOutput>,
        val wallets: List<CreateWalletOutput>,
        val psbts: List<PsbtJsonOutput>
    )

    data class MasterKeyOutput(
        val key: PrivateMasterKey,
        val private_file: String,
        val public_file: String?,
        val public_qr_files: List<String>
    )

    data class PrivateMasterKey(
        val name: String,
        val xpub: String,
        val xprv: String,
        val mnemonic: String?,
        val dice: Dice?,
        val fingerprint: String
    )

    data class Dice(val launches: String, val faces: Int, val value: String)

    data class CreateWalletOutput(
        val wallet_file: String,
        val wallet: WalletJson,
        val qr_files: List<String>
    )

    data class WalletJson(
        val name: String,
        val descriptor_main: String,
        val descriptor_change: String,
        val fingerprints: List<String>,
        val required_sig: Int,
        val created_at_height: Int,
        val daemon_opts: DaemonOpts?
    )

    data class DaemonOpts (
        val url: String,
        val cookie_file: String
    )

    data class PsbtJson (
        val name: String,
        val psbt: String,
        val fee: Double,
        val changepos: Int
    )

    data class PsbtJsonOutput (
        val signatures: String,
        val psbt: PsbtJson,
        val file: String,
        val qr_files: List<String>,
        val unsigned_txid: String
    )

    data class TxIn (
        val outpoint: String,
        val signatures: List<String>,
        val value: String,
        val wallet_with_path: String?
    )

    data class TxOut (
        val address: String,
        val value: String,
        val wallet_with_path: String?

    )

    data class Size (
        val unsigned: Int,
        val estimated: Int,
        val psbt: Int
    )

    data class Fee (
        val absolute_fmt: String,
        val absolute: Long,
        val rate : Double
    )

    data class PsbtPrettyPrint (
        val inputs: List<TxIn>,
        val outputs: List<TxOut>,
        val size: Size,
        val fee: Fee,
        val info: List<String>,
        val psbt_file: String,
        val balances: String
    )

    data class GetAddressOutput (
        val address: String,
        val path: String
    )

    private external fun call(json: String): String

    private fun callJson(json: String): JsonNode {
        val strResult = call(json)
        val jsonResult = mapper.readTree(strResult)
        if (jsonResult.has("error")) {
            throw RustException(jsonResult["error"].toString())
        }
        return jsonResult
    }

    fun list(datadir: String, kind: String): ListOutput {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("kind", kind)
        val req = JsonRpc("list", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return mapper.convertValue(json, ListOutput::class.java)
    }

    fun random(datadir: String, keyName: String): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key_name", keyName)
        node.put("qr_version", 14)
        val req = JsonRpc("random", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun dice(datadir: String, keyName: String, faces: String, launches: ArrayList<Int>): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key_name", keyName)
        node.put("qr_version", 14)
        node.put("faces", "_$faces")
        node.put("bits", "_256")
        val array = node.putArray("launches")
        for (el in launches) {
            array.add(el)
        }
        val req = JsonRpc("dice", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun mergeQrs(datadir: String, qrs_bytes: List<String>): String {
        val node = JsonNodeFactory.instance.arrayNode()
        for (bytes in qrs_bytes) {
            node.add(bytes)
        }
        val req = JsonRpc("merge_qrs", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString).asText()
    }

    fun importWallet(datadir: String, wallet: WalletJson) {
        val node: JsonNode = mapper.valueToTree(wallet);
        val req = JsonRpc("import_wallet", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        callJson(reqString)
    }

    fun sign(datadir: String, key: String, wallet: String, psbt: String): PsbtPrettyPrint {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key", key)
        node.put("wallet_descriptor_file", wallet)
        node.put("psbt_file", psbt)
        node.put("total_derivations", 100)
        node.put("qr_version", 14)
        val req = JsonRpc("sign", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun restore(datadir: String, key: String, nature: String, value: String): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key_name", key)
        node.put("nature", nature)
        node.put("value", value)
        node.put("qr_version", 14)
        val req = JsonRpc("restore", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun print(datadir: String, psbt_file: String): PsbtPrettyPrint {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("psbt_file", psbt_file)
        val req = JsonRpc("print", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return mapper.convertValue(json, PsbtPrettyPrint::class.java)
    }

    fun savePSBT(datadir: String, psbt: String, encoding: String) {  //TODO enum for encoding
        val node = JsonNodeFactory.instance.objectNode()
        node.put("qr_version", 14)
        val psbtNode = JsonNodeFactory.instance.objectNode()
        psbtNode.put("t", encoding)
        psbtNode.put("c", psbt)
        node.set("psbt", psbtNode)
        val req = JsonRpc("save_psbt", datadir, Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        callJson(reqString)
    }

    fun deriveAddress(walletDescriptor: String, addressIndex: Int): GetAddressOutput {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("descriptor", walletDescriptor)
        node.put("index", addressIndex)
        val req = JsonRpc("derive_address", "", Network.TYPE, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return mapper.convertValue(json, GetAddressOutput::class.java)
    }

}

class RustException(message:String): Exception(message)

