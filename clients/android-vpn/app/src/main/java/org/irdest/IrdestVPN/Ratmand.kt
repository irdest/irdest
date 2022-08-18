package org.irdest.IrdestVPN

import android.util.Log

class Ratmand {
    private val TAG = Ratmand::class.java.simpleName

    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private external fun ratrun(test_str: String): String

    var arive = true

    fun runRatmand(test_string: String?) {
        Log.d(TAG, "runRatmand: runRatmand is called")

        arive = true

        val ratmandThread = object : Thread() {
            override fun run() {
                Log.d(TAG, "run_ratmand(Ratmand): Current thread = "
                        + Thread.currentThread())

                ratrun("test")

                Log.d(TAG, "run: ratmand is stopped")
            }
        }
        ratmandThread.isDaemon = true
        ratmandThread.stackTrace
        ratmandThread.start()
    }

    fun stopRatmand() {
        Log.d(TAG, "stopRatmand: Stop Ratmand is called")
        arive = false
    }
}