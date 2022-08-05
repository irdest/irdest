package org.irdest.IrdestVPN

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import android.view.View
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    private val TAG = MainActivity::class.java.simpleName

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
    }

    private var resultLauncher = registerForActivityResult(
            ActivityResultContracts.StartActivityForResult()) { result ->
            if (result.resultCode == Activity.RESULT_OK) {
            startService(getService())
        }
    }

    fun startVpn(view: View) {
        Log.d(TAG, "startVpn: Connect button is clicked")
        // Ask for permission
        var intent = VpnService.prepare(this)
        if (intent != null) {
            resultLauncher.launch(intent)
        } else {
            startService(getService())
        }
    }

    fun stopVpn(view: View) {
        Log.d(TAG, "stopVpn: Disconnect button is clicked")
        startService(getService()
            .putExtra("ACTION", "disconnect"))
    }

    private fun getService() : Intent {
        return Intent(this, IrdestVpnService::class.java)
    }
}