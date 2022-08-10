package org.irdest.IrdestVPN

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class Ratmand: ViewModel() {
    private val TAG = Ratmand::class.java.simpleName
    // JNI constructs the name of function that it will call is
    // Java_<domain>_<class>_<methodname> (Java_org_irdest_ratman_Ratmand_ratrun).
    private val defaultDispatcher : CoroutineDispatcher = Dispatchers.Default

    private external fun ratrun(test_str: String): String

    var arive = true

    fun runRatmand(test_string: String?) {
        arive = true

        viewModelScope.launch(defaultDispatcher) {
            Log.d(TAG, "run_ratmand: Current thread = "
                    + Thread.currentThread())

            while(arive) {
                delay(2000)
                var returned = ratrun("test")
                Log.d(TAG, "run_ratmand: ratmand is running ....")
            }
        }
    }

    fun stopRatmand() {
        Log.d(TAG, "stopRatmand: Stop Ratmand is called")
        arive = false
    }
}