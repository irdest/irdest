package org.irdest.IrdestVPN

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import org.irdest.IrdestVPN.R

class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("ratman-client")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

    }
}