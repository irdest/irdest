package org.irdest.IrdestVPN

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.view.View
import android.widget.Toast
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import org.irdest.IrdestVPN.utils.createRatmandDataFileIfNotExist

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

        createRatmandDataFileIfNotExist(this)
    }

    fun startVpn(view: View) {
        // Prepare the app to become the user's current VPN service.
        // If user hasn't given permission `VpnService.prepare()` returns an activity intent.
        VpnService.prepare(this)
            ?.let { permissionActivityLauncher.launch(it) }
            ?:run { startService(getActionSettedServiceIntent()) }
    }

    private val permissionActivityLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            startService(getActionSettedServiceIntent())
        } else {
           Toast.makeText(this, R.string.permission_denied, Toast.LENGTH_LONG).show()
        }
    }

    fun stopVpn(view: View) {
        startService(getActionSettedServiceIntent(IrdestVpnService.ACTION_DISCONNECT))
    }

    private fun getActionSettedServiceIntent(
        action: String = IrdestVpnService.ACTION_CONNECT) : Intent {
        return Intent(this, IrdestVpnService::class.java)
            .setAction(action)
    }
}