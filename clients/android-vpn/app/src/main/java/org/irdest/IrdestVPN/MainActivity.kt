package org.irdest.IrdestVPN

import android.os.Bundle
import android.view.View
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

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