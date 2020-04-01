package it.casatta

import android.app.Application


class FirmaApplication : Application() {
    companion object {
        init {
            System.loadLibrary("firma")
        }
    }
}