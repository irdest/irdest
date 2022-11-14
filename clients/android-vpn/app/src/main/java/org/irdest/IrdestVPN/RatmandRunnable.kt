package org.irdest.IrdestVPN

import android.util.Log

class RatmandRunnable : Runnable {
    private val TAG = RatmandRunnable::class.java.simpleName

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