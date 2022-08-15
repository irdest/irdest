package org.irdest.IrdestVPN

import android.util.Log
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers

class Ratmand {
    private val TAG = Ratmand::class.java.simpleName
    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private val defaultDispatcher : CoroutineDispatcher = Dispatchers.Default

    private external fun ratrun(test_str: String): String

    var arive = true

    fun runRatmand(test_string: String?) {
        arive = true

        // Create new thread
        val ratmandThread = object : Thread() {
            override fun run() {
                Log.d(TAG, "run_ratmand(Ratmand): Current thread = "
                        + Thread.currentThread())

                ratrun("test")

                // Looping
                while(arive) {
                    Thread.sleep(2000)
                    Log.d(TAG, "run_ratmand: ratmand is running ....")
                }
            }
        }
        ratmandThread.isDaemon = true
        ratmandThread.start()
    }

    fun stopRatmand() {
        Log.d(TAG, "stopRatmand: Stop Ratmand is called")
        arive = false
    }
}