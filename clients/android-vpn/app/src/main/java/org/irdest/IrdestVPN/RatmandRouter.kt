package org.irdest.IrdestVPN

import android.util.Log
import org.irdest.IrdestVPN.utils.ConnectionState

class RatmandRouter : Runnable {
    private val TAG = RatmandRouter::class.java.simpleName

    // Jni(ffi)
    private external fun receiveLog()
    private external fun runRouter()
    private external fun registerUser()

    override fun run() {
        try {
            // receiveLog()
            runRouter()
        } catch (e: Exception) {
            when(e) {
                is InterruptedException -> Log.i(TAG, "run: Stop ratmand")
                else -> {
                    Log.e(TAG, "run: Error", e)
                }
            }
        }
    }
}