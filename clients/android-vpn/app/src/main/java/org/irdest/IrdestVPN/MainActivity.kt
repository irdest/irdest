package org.irdest.IrdestVPN

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import org.irdest.IrdestVPN.R

class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("ratman")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val r = Ratmand()
        val test_op = r.run_ratmand("From android")

        (findViewById<View>(R.id.test_view) as TextView).text = test_op
    }
}