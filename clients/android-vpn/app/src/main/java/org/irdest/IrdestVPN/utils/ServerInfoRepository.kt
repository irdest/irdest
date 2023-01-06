package org.irdest.irdestVPN.utils

import android.annotation.SuppressLint
import android.content.Context
import org.irdest.irdestVPN.models.ServerInfo

enum class ServerInfoPrefs(val key: String) {
    SERVER_ADDRESS("server_address"),
    SERVER_PORT("server_port"),
    SHARED_SECRET("shared_secret"),
}

class ServerInfoRepository(context: Context) {

    private val sharedPrefs =
        context.applicationContext
            .getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)

    fun getServerInfo(): ServerInfo {
        val serverAddress =
            sharedPrefs.getString(ServerInfoPrefs.SERVER_ADDRESS.key, "") ?: ""
        val port =
            sharedPrefs.getString(ServerInfoPrefs.SERVER_PORT.key, "") ?: ""
        val sharedSecret =
            sharedPrefs.getString(ServerInfoPrefs.SHARED_SECRET.key, "") ?: ""

        return ServerInfo(serverAddress, port, sharedSecret)
    }

    fun saveServerInfo(serverInfo: ServerInfo) {
        with(sharedPrefs.edit()) {
            putString(ServerInfoPrefs.SERVER_ADDRESS.key, serverInfo.serverAddress)
            putString(ServerInfoPrefs.SERVER_PORT.key, serverInfo.serverPort)
            putString(ServerInfoPrefs.SHARED_SECRET.key, serverInfo.sharedSecret)

            commit()
        }
    }

    companion object {
        private const val PREFERENCES_NAME = "server_info_prefs"

        @SuppressLint("StaticFieldLeak")
        @Volatile
        private var currentInstance: ServerInfoRepository? = null

        fun getInstance(context: Context): ServerInfoRepository {
            synchronized(this) {
                currentInstance?.let {
                    return it
                }
            }

            val newInstance = ServerInfoRepository(context)
            currentInstance = newInstance
            return newInstance
        }
    }
}