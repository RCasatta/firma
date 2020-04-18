package it.casatta

import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.JsonNodeFactory
import com.fasterxml.jackson.module.kotlin.KotlinModule

class Rust {
    val mapper = ObjectMapper().registerModule(KotlinModule())

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
        val seed: Seed?,
        val dice: Dice?,
        val fingerprint: String
    )

    data class Seed(val hex: String, val bech32: String, val network: String)
    data class Dice(val launches: String, val faces: Int)

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
        val created_at_height: Int
    )

    data class PsbtJson (
        val name: String,
        val psbt: String,
        val fee: Double,
        val changepos: Int
    )

    data class PsbtJsonOutput (
        val psbt: PsbtJson,
        val file: String,
        val qr_files: List<String>
    )

    data class TxInOut (
        val outpoint: String?,
        val address: String?,
        val value: String,
        val path: String,
        val wallet: String?
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
        val inputs: List<TxInOut>,
        val outputs: List<TxInOut>,
        val size: Size,
        val fee: Fee,
        val info: List<String>,
        val psbt_file: String,
        val balances: String
    )

    external fun call(json: String): String

    fun callJson(json: String): JsonNode {
        val strResult = call(json)
        val jsonResult = mapper.readTree(strResult)
        if (jsonResult.has("error")) {
            throw RustException(jsonResult["error"].toString())
        }
        return jsonResult
    }

    fun list(datadir: String, network: String, kind: String): ListOutput {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("kind", kind)
        val req = JsonRpc("list", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        val output = mapper.convertValue(json, ListOutput::class.java)
        return output
    }

    fun random(datadir: String, network: String, keyName: String): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key_name", keyName)
        node.put("qr_version", 14)
        val req = JsonRpc("random", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString)
    }

    fun merge_qrs(datadir: String, network: String, qrs_bytes: List<String>): String {
        val node = JsonNodeFactory.instance.arrayNode()
        for (bytes in qrs_bytes) {
            node.add(bytes)
        }
        val req = JsonRpc("merge_qrs", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        return callJson(reqString).asText()
    }

    fun create_qrs(file: String, network: String) {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("path", file)
        node.put("qr_version", 14)
        val req = JsonRpc("create_qrs", "", network, node)
        val reqString = mapper.writeValueAsString(req)
        callJson(reqString)
    }

    fun sign(datadir: String, network: String, key: String, wallet: String, psbt: String): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key", key)
        node.put("wallet_descriptor_file", wallet)
        node.put("psbt_file", psbt)
        node.put("total_derivations", 100)
        node.put("qr_version", 14)
        val req = JsonRpc("sign", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return json
    }

    fun restore(datadir: String, network: String, key: String, nature: String, value: String): JsonNode {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("key_name", key)
        node.put("nature", nature)
        node.put("value", value)
        node.put("qr_version", 14)
        val req = JsonRpc("restore", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        return json
    }

    fun print(datadir: String, network: String, psbt_file: String): PsbtPrettyPrint {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("psbt_file", psbt_file)
        val req = JsonRpc("print", datadir, network, node)
        val reqString = mapper.writeValueAsString(req)
        val json = callJson(reqString)
        val output = mapper.convertValue(json, PsbtPrettyPrint::class.java)
        return output
    }
}

class RustException(message:String): Exception(message)

