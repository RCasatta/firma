package it.casatta.json

class Options {

    data class ListOptions(
        val kind: Data.Kind,
        val encryption_keys: List<Data.StringEncoding>
    )

    data class RandomOptions(
        val key_name: String,
        val qr_version: Int,
        val encryption_key: Data.StringEncoding?
    )

    data class DiceOptions(
        val key_name: String,
        val qr_version: Int,
        val encryption_key: Data.StringEncoding?,
        val faces: Data.Base,
        val bits: Data.Bits,
        val launches: List<Int>
    )

    data class SignOptions(
        val key: String,
        val total_derivations: Int,
        val wallet_descriptor_file: String,
        val qr_version: Int,
        val psbt_file: String,
        val allow_any_derivation: Boolean,
        val encryption_key: Data.StringEncoding?
    )

    data class RestoreOptions(
        val key_name: String,
        val nature: Data.Nature,
        val qr_version: Int,
        val encryption_key: Data.StringEncoding?,
        val value: String
    )

    data class PrintOptions(
        val psbt_file: String
    )

    data class SavePSBTOptions(
        val psbt: Data.StringEncoding,
        val qr_version: Int
    )

    data class DeriveAddressOpts(
        val descriptor: String,
        val index: Int
    )
}