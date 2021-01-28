package it.casatta.json

import android.util.Base64
import com.fasterxml.jackson.annotation.JsonValue

class Data {

    companion object {
        private fun hexToByte(hexString: String): Byte {
            val firstDigit = toDigit(hexString[0])
            val secondDigit = toDigit(hexString[1])
            return ((firstDigit shl 4) + secondDigit).toByte()
        }

        private fun toDigit(hexChar: Char): Int {
            val digit: Int = Character.digit(hexChar, 16)
            if (digit == -1) {
                throw IllegalArgumentException(
                    "Invalid Hexadecimal Character: $hexChar")
            }
            return digit
        }

        private fun decodeHexString(hexString: String): ByteArray {
            if (hexString.length % 2 == 1) {
                throw IllegalArgumentException(
                    "Invalid hexadecimal String supplied.")
            }
            val bytes = ByteArray(hexString.length / 2)
            var i = 0
            while (i < hexString.length ) {
                bytes[i / 2] = hexToByte(hexString.substring(i, i + 2))
                i += 2
            }
            return bytes
        }

        fun decodeStringEncoding(data: StringEncoding): ByteArray {
            return when(data.t) {
                Encoding.HEX -> decodeHexString(data.c)
                Encoding.PLAIN -> data.c.toByteArray(Charsets.UTF_8)
                Encoding.BASE64 -> Base64.decode(data.c, Base64.DEFAULT)
                Encoding.BECH32 -> error("Assertion failed, bech32 not supported")
            }
        }

        fun encodeStringEncodingHex(data: ByteArray): StringEncoding {
            return StringEncoding(Encoding.HEX, data.toHexString())
        }
    }

    data class JsonRpc(
        val method: String,
        val context: Context,
        val args: Any
    )

    data class Context(
        val datadir: String,
        val network: String,
        val encryption_key: StringEncoding?
    )

    data class ListOutput(
        val master_secrets: List<PrivateMasterKey>,
        val wallets: List<WalletJson>,
        val psbts: List<PsbtJson>
    )

    data class Identifier(
        val kind: Kind,
        val name: String,
        val network: String
    )

    data class PrivateMasterKey(
        val id: Identifier,
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

    data class WalletJson(
        val id: Identifier,
        val descriptor: String,
        val fingerprints: List<String>,
        val required_sig: Int,
        val created_at_height: Int
    )

    data class WalletSignature(
        val id: Identifier,
        val xpub: String,
        val address: String,
        val signature: String
    )

    data class PsbtJson(
        val id: Identifier,
        val psbt: String,
        val fee: Double,
        val changepos: Int
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

    data class EncodedQrs(
        val qrs: List<StringEncoding>
    )

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
        BECH32("bech32"),
        PLAIN("plain")
    }

    enum class Kind(@JsonValue val code: String) {
        WALLET("Wallet"),
        WALLET_INDEXES("WalletIndexes"),
        WALLET_SIGNATURE("WalletSignature"),
        MASTER_SECRET("MasterSecret"),
        DESCRIPTOR_PUBLIC_KEY("DescriptorPublicKey"),
        PSBT("PSBT")
    }
}

fun ByteArray.toHexString() = joinToString("") { "%02x".format(it) }