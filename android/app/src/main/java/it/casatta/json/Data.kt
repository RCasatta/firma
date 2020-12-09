package it.casatta.json

import com.fasterxml.jackson.annotation.JsonValue

class Data {
    data class JsonRpc(
        val method: String,
        val datadir: String,
        val network: String,
        val args: Any
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

    data class Dice(
        val launches: String,
        val faces: Int,
        val value: String
    )

    data class CreateWalletOutput(
        val wallet_file: String,
        val wallet: WalletJson,
        val qr_files: List<String>
    )

    data class WalletJson(
        val name: String,
        val descriptor: String,
        val fingerprints: List<String>,
        val required_sig: Int,
        val created_at_height: Int,
        val daemon_opts: DaemonOpts?
    )

    data class DaemonOpts(
        val url: String,
        val cookie_file: String
    )

    data class PsbtJson(
        val name: String,
        val psbt: String,
        val fee: Double,
        val changepos: Int
    )

    data class PsbtJsonOutput(
        val signatures: String,
        val psbt: PsbtJson,
        val file: String,
        val qr_files: List<String>,
        val unsigned_txid: String
    )

    data class TxIn(
        val outpoint: String,
        val signatures: List<String>,
        val value: String,
        val wallet_with_path: String?
    )

    data class TxOut(
        val address: String,
        val value: String,
        val wallet_with_path: String?

    )

    data class Size(
        val unsigned: Int,
        val estimated: Int,
        val psbt: Int
    )

    data class Fee(
        val absolute_fmt: String,
        val absolute: Long,
        val rate: Double
    )

    data class PsbtPrettyPrint(
        val inputs: List<TxIn>,
        val outputs: List<TxOut>,
        val size: Size,
        val fee: Fee,
        val info: List<String>,
        val psbt_file: String,
        val balances: String
    )

    data class GetAddressOutput(
        val address: String,
        val path: String
    )

    data class StringEncoding(
        val t: Encoding,
        val c: String
    )

    enum class Kind(@JsonValue val code: String) {
        WALLET("wallets"),
        KEY("keys"),
        PSBT("psbts")
    }

    enum class Base(@JsonValue val code: String) {
        _2("2"),
        _4("4"),
        _6("6"),
        _8("8"),
        _12("12"),
        _20("20")
    }

    enum class Bits(@JsonValue val code: String) {
        _128("128"),
        _192("192"),
        _256("256")
    }

    enum class Nature(@JsonValue val code: String) {
        XPRV("Xprv"),
        MNEMONIC("Mnemonic")
    }

    enum class Encoding(@JsonValue val code: String) {
        BASE64("base64"),
        HEX("hex"),
        BECH32("bech32")
    }

}