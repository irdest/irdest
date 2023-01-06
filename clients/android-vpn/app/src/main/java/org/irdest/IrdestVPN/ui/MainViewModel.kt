package org.irdest.irdestVPN.ui

import android.app.Activity
import android.content.Intent
import android.os.Looper
import android.os.Message
import android.os.Messenger
import androidx.activity.result.ActivityResult
import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import org.irdest.irdestVPN.models.*
import org.irdest.irdestVPN.models.ConnectionState.*
import org.irdest.irdestVPN.utils.MessageHandler
import org.irdest.irdestVPN.utils.ServerInfoRepository
import org.irdest.irdestVPN.vpn.VpnServiceStarter
import java.util.*

class MainViewModel(
    private val serverInfoRepository: ServerInfoRepository,
    private val vpnServiceStarter: VpnServiceStarter
) : ViewModel(), MessageHandler.MessageReceiver {

    // Load the last used server info.
    private val serverInfo: ServerInfo = serverInfoRepository.getServerInfo()

    private val vpnState: MutableLiveData<VpnState> = MutableLiveData(VpnState())
    private var permissionIntent: MutableLiveData<Intent?> = MutableLiveData(null)

    fun observeVpnState(): MutableLiveData<VpnState> = vpnState
    fun observePermissionIntent(): MutableLiveData<Intent?> = permissionIntent

    fun onConnectBtnClick() {
        try {
            // Validate user input and save to shared preferences.
            if (serverInfo.isValid()) {
                serverInfoRepository.saveServerInfo(serverInfo)
            }

            when (val intent = vpnServiceStarter.prepareVpnService()) {
                null -> startVpnService()
                else -> permissionIntent.value = intent
            }
        } catch (e: InputMismatchException) {
            setVpnState(ERROR, e.message.toString())
        }
    }

    private fun startVpnService() {
        setVpnState(CONNECTING)
        vpnServiceStarter.start(getMessenger())
    }

    private fun getMessenger() =
        Messenger(MessageHandler(Looper.getMainLooper(), this))

    fun handlePermissionActivityResult(result: ActivityResult) {
        permissionIntent.value = null

        when (result.resultCode) {
            Activity.RESULT_OK -> startVpnService()
            else -> setVpnState(ERROR, "Permission denied.")
        }
    }

    fun onDisconnectBtnClick() {
        vpnServiceStarter.stop()
    }

    override fun onReceiveMessage(msg: Message) {
        // Handle received message from the VPN Service.
        setVpnState(getConnectionStateFromCode(msg.what), msg.obj.toString())
    }

    private fun setVpnState(state: ConnectionState, msg: String = "") {
        vpnState.value = VpnState(state, msg)
    }

    // Two-way data binding.
    fun setServerAddress(address: String) {
        serverInfo.serverAddress = address
    }

    fun setServerPort(port: String) {
        serverInfo.serverPort = port
    }

    fun setSharedSecret(secret: String) {
        serverInfo.sharedSecret = secret
    }

    fun getServerAddress(): String = serverInfo.serverAddress

    fun getServerPort(): String = serverInfo.serverPort

    fun getSharedSecret(): String = serverInfo.sharedSecret

    class MainViewModelFactory(
        private val serverInfoRepository: ServerInfoRepository,
        private val vpnServiceStarter: VpnServiceStarter,
    ) : ViewModelProvider.Factory {

        @Suppress("UNCHECKED_CAST")
        override fun <T : ViewModel> create(modelClass: Class<T>): T {
            if (modelClass.isAssignableFrom(MainViewModel::class.java)) {
                return MainViewModel(serverInfoRepository, vpnServiceStarter) as T
            }

            throw IllegalArgumentException("Unknown ViewModel class")
        }
    }
}