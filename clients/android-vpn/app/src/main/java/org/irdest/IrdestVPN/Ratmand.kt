package org.irdest.IrdestVPN

import android.util.Log
import java.lang.RuntimeException
import java.util.concurrent.atomic.AtomicReference
import kotlin.concurrent.thread

class Ratmand : Runnable {
    private val TAG = Ratmand::class.java.simpleName

    private external fun receiveLog()
    private external fun ratrun()

    override fun run() {
        while (true) {
            try {
                 receiveLog()
                 ratrun()
            } catch(e: InterruptedException) {
                Log.d(TAG, "run: Stop ratmand thread ", e)
            }
            throw RuntimeException();
        }
    }
}