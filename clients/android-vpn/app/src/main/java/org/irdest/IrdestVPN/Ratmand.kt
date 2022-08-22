package org.irdest.IrdestVPN

import android.util.Log
import java.lang.RuntimeException

class Ratmand {
    private val TAG = Ratmand::class.java.simpleName
    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private external fun ratrun(test_str: String): String

    val handler = object : Thread.UncaughtExceptionHandler {
        override fun uncaughtException(t: Thread, e: Throwable) {
            Log.e(TAG, "uncaughtException: ratmand thread throw exception", e)
        }
    }

    fun runRatmand(test_string: String?) {
        val ratmandThread = object : Thread() {
            override fun run() {
                try {
                    ratrun("test")
                } catch(e: InterruptedException){
                    Log.e(TAG, "run: ratmand thread interrupted ", e)
                }
                // Catch uncaught exception
                Log.d(TAG, "run: ratmand is stopped")
                throw RuntimeException();
            }
        }
        ratmandThread.stackTrace
        ratmandThread.isDaemon = true
        ratmandThread.setUncaughtExceptionHandler(handler)

        ratmandThread.start()
    }

    fun stopRatmand() {
        //TODO: Stop ratmand
    }
}