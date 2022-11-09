package org.irdest.IrdestVPN

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.view.View
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import org.irdest.IrdestVPN.utils.createFileIfNotExist

class MainActivity : AppCompatActivity() {
    private val TAG = MainActivity::class.java.simpleName

    companion object {
        init {
            System.loadLibrary("ratman_android")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Check if data file for ratmand is exist if not create one.
        createFileIfNotExist(this, "ratmand_data")
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