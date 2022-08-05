package org.irdest.IrdestVPN

import android.app.*
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.annotation.RequiresApi
import java.io.FileInputStream
import java.io.FileOutputStream

/* Local tun interface address*/
// IpV4
const val VPN_ADDRESS = "10.0.0.2"
const val VPN_ROUTE = "0.0.0.0"
const val VPN_DNS = "8.8.8.8"
// IpV6
const val VPN_ADDRESS_V6 = "2001:db8::1"
const val VPN_ROUTE_V6 = "::" // Intercept all

class IrdestVpnService : VpnService() {
    private val TAG = IrdestVpnService::class.java.simpleName

    private val NOTIFICATION_CHANNEL_ID = "IrdestVpn"
    private lateinit var clientIntent: Intent

    private lateinit var vpnInterface: ParcelFileDescriptor
    private lateinit var connection: Connection


    override fun onCreate() {
        super.onCreate()
        // When user click the foreground notification, this activity will be opened.
        clientIntent = Intent(this, MainActivity::class.java)

        // Start foreground service
        startForegroundService()
        updateForegroundNotification(R.string.connecting)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.getStringExtra("ACTION") == "disconnect") {
            // onDestroy()
            disconnect()
            return Service.START_NOT_STICKY
        } else {
            connect()
            return Service.START_STICKY
        }
    }

    @Throws(NullPointerException::class)
    private fun openTun() {
        vpnInterface = Builder()
            .addAddress(VPN_ADDRESS_V6, 64)
            .addRoute(VPN_ROUTE_V6, 0)
            .setSession(TAG)
            .establish()!!

        Log.d(TAG, "openTun: New tun interface is created")
    }

    private fun connect() {
        try {
            openTun()
            // Receive from local and send to remote network.
            val inputStream =
                FileInputStream(vpnInterface.fileDescriptor)
                .channel
            // Receive from remote and send to local network.
            val outputStream =
                FileOutputStream(vpnInterface.fileDescriptor)
                .channel

            connection = Connection(inputStream, outputStream)
            // TODO: Connect to remote server & protect tunnel
        }
        catch (e: Exception) {
            Log.e(TAG, "connect: Failed to open tun interface", e)
        }
        finally {
            connection.runForever()
            updateForegroundNotification(R.string.connected)
        }
    }

    private fun disconnect() {
        connection.disconnect()
        vpnInterface?.close()
        stopForeground(true)
        stopSelf()
        Log.i(TAG, "stopVPN: Vpn service is stopped")
    }

    @RequiresApi(Build.VERSION_CODES.O)
    private fun startForegroundService() {
        (getSystemService(NOTIFICATION_SERVICE) as NotificationManager)
            .createNotificationChannel(
                NotificationChannel(
                    NOTIFICATION_CHANNEL_ID,
                    NOTIFICATION_CHANNEL_ID,
                    NotificationManager.IMPORTANCE_DEFAULT
                )
            )

        applicationContext.startForegroundService(clientIntent)
    }

    @RequiresApi(Build.VERSION_CODES.O)
    private fun updateForegroundNotification(msg: Int) {
        val configureIntent =
            PendingIntent.getActivity(
                this,
                0,
                clientIntent,
                PendingIntent.FLAG_UPDATE_CURRENT)

        startForeground(
            1,
            Notification.Builder(this, NOTIFICATION_CHANNEL_ID)
                .setContentTitle("Irdest Vpn")
                .setContentText(getString(msg))
                .setSmallIcon(R.drawable.ic_launcher_foreground)
                .setContentIntent(configureIntent)
                .build()
        )
    }
}