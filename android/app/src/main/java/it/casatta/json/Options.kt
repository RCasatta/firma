package it.casatta.json

class Options {

    data class ListOptions(
        val kind: Data.Kind
    )

    data class WalletNameOptions(
        val wallet_name: String
    )

    data class RandomOptions(
        val key_name: String
    )

    data class QrOptions(
        val qr_content: Data.StringEncoding,
        val version: Int
    )

    data class QrMergeOptions(
        val qrs_content: List<Data.StringEncoding>
    )

    data class DiceOptions(
        val key_name: String,
        val faces: Data.Base,
        val bits: Data.Bits,
        val launches: List<Int>
    )

    data class SignOptions(
        val key_name: String,
        val wallet_name: String,
        val psbt_name: String,
        val total_derivations: Int,
        val allow_any_derivations: Boolean
    )

    data class RestoreOptions(
        val key_name: String,
        val nature: Data.Nature,
        val value: String
    )

    data class PrintOptions(
        val psbt_file: String?,
        val psbt_base64: String?,
        val psbt_name: String?,
        val verify_wallets_signatures: Boolean
    )

    data class SavePsbtOptions(
        val psbt: Data.StringEncoding
    )

    data class DeriveAddressOpts(
        val descriptor: String,
        val index: Int
    )

    data class VerifyWalletResult(
        val descriptor: String,
        val signature: Data.WalletSignature,
        val verified: Boolean
    )

    data class ExportOptions(
        val kind: Data.Kind,
        val name: String
    )

}