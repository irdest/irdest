package org.irdest.IrdestVPN

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.util.Log
import android.view.View
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import java.io.File
import java.util.concurrent.atomic.AtomicReference

class MainActivity : AppCompatActivity() {
    private val TAG = MainActivity::class.java.simpleName

    private val DATA_FILE = "users.json"

    companion object {
        init {
            System.loadLibrary("ratman_android")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Check if data file is exist if not create one.
        // kinda first thing to do: move this config handling to Utils
        val data_file = File(applicationContext.filesDir, DATA_FILE)
        if (!data_file.exists()) {
            Log.i(TAG, "onCreate: Data file is not existed create new file")
            val isCreated = data_file.createNewFile()
            Log.i(TAG, "onCreate: File is created = " + isCreated)
        }
    }

    fun startVpn(view: View) {
        // Prepare the app to become the user's current VPN service.
        // If user hasn't given permission `VpnService.prepare()` returns an activity intent.
        VpnService.prepare(this)?.let { permissionActivityLauncher.launch(it) }
            ?: run { startService(getService().setAction(IrdestVpnService.ACTION_CONNECT))}
    }

    fun stopVpn(view: View) {
        startService(getService().setAction(IrdestVpnService.ACTION_DISCONNECT))
    }

    private val permissionActivityLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            startService(getService().setAction(IrdestVpnService.ACTION_CONNECT))
        }
    }

    private fun getService() : Intent {
        return Intent(this, IrdestVpnService::class.java)
    }
}