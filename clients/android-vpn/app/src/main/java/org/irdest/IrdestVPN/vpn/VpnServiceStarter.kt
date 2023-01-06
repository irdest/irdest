package org.irdest.irdestVPN.vpn

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Messenger

class VpnServiceStarter(private val context: Context) {

    fun prepareVpnService(): Intent? {
        return VpnService.prepare(context)
    }

    fun start(messenger: Messenger) {
        val intent = getServiceIntent()
        intent.action = IrdestVpnService.ACTION_CONNECT
        intent.putExtra(IrdestVpnService.MESSENGER_EXTRA_NAME, messenger)

        context.startService(intent)
    }

    fun stop() {
        val intent = getServiceIntent()
        intent.action = IrdestVpnService.ACTION_DISCONNECT

        context.startService(intent)
    }

    private fun getServiceIntent() =
        Intent(context.applicationContext, IrdestVpnService::class.java)

    companion object {
        @SuppressLint("StaticFieldLeak")
        @Volatile
        private var currentInstance: VpnServiceStarter? = null

        fun getInstance(context: Context): VpnServiceStarter {
            synchronized(this) {
                currentInstance?.let {
                    return it
                }
            }

            val newInstance = VpnServiceStarter(context)
            currentInstance = newInstance
            return newInstance
        }
    }
}