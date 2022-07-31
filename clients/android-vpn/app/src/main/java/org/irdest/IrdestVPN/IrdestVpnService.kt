package org.irdest.IrdestVPN

import android.annotation.SuppressLint
import android.app.*
import android.content.ContextWrapper
import android.content.Intent
import android.net.VpnService
import android.os.*
import android.util.Log
import android.widget.Toast
import androidx.annotation.RequiresApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.selects.whileSelect
import java.io.FileInputStream
import java.io.FileOutputStream
import java.nio.ByteBuffer
import java.nio.channels.FileChannel

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

    private var vpnInterface: ParcelFileDescriptor? = null

    // Receive from local and send to remote network.
    private var inputStream: FileChannel? = null
    // Receive from remote and send to local network.
    private var outputStream: FileChannel? = null

    override fun onCreate() {
        super.onCreate()

        // Set foreground service
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            updateForegroundNotification(R.string.connecting)
        }
        // Open local tunnel
        openTun()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.getStringExtra("ACTION") == "disconnect") {
            disconnect()
            return Service.START_NOT_STICKY
        } else {
            connect()
            return Service.START_STICKY
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        disconnect()
    }

    private fun openTun() {
        vpnInterface = Builder()
            .addAddress(VPN_ADDRESS_V6, 64)
            .addRoute(VPN_ROUTE_V6, 0)
            .setSession(TAG)
            .establish()

        Log.d(TAG, "openTun: New tun interface created")
    }

    private fun connect() = runBlocking {
        // Receive from local and send to remote network.
        inputStream = FileInputStream(vpnInterface!!.fileDescriptor)
            .channel
        // Receive from remote and send to local network.
        outputStream = FileOutputStream(vpnInterface!!.fileDescriptor)
            .channel

        launch {
            mainLoop(inputStream!!, outputStream!!)
        }

        Log.d(TAG, "vpnRunLoop: Main loop stopped")
    }

    private suspend fun mainLoop(
        input: FileChannel, output: FileChannel) = runBlocking {
        Log.d(TAG, "connect: Running main loop...")

        var alive = true

        launch(Dispatchers.IO) {
            Log.d(TAG, "mainLoop: ### Main loop couroutine starts")
            loop@ while (alive) {
                val buffer = ByteBuffer.allocate(1024)

                if (input.read(buffer) <= 0) {
                    delay(100)
                    continue@loop
                }

                // Received packet from local.
                Log.d(TAG, "vpnRunLoop: Received packet from loacl")
            }
        }
        alive = false
    }

    private fun disconnect() {
        inputStream?.close()
        outputStream?.close()
        vpnInterface?.close()
        stopForeground(true)
        stopSelf()
        Log.i(TAG, "stopVPN: Vpn service stopped")
    }

    // Start foreground service
    @RequiresApi(Build.VERSION_CODES.O)
    private fun updateForegroundNotification(msg: Int) {
        // Init NotificationManager
        val NOTIFICATION_CHANNEL_ID = "IrdestVpn"
        val notificationMng = getSystemService(NOTIFICATION_SERVICE) as NotificationManager
        notificationMng.createNotificationChannel(
            NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                NOTIFICATION_CHANNEL_ID,
                NotificationManager.IMPORTANCE_DEFAULT
            )
        )

        val mainIntent = Intent(this, MainActivity::class.java)
        applicationContext.startForegroundService(mainIntent)

        val configureIntent =
            PendingIntent.getActivity(
                this,
                0,
                mainIntent,
                PendingIntent.FLAG_UPDATE_CURRENT)


        startForeground(
            1,
            Notification.Builder(this, NOTIFICATION_CHANNEL_ID)
                .setContentTitle("Irdest Vpn")
                .setContentText(getString(msg))
                .setTicker("Message Ticker")
                .setSmallIcon(R.drawable.ic_launcher_foreground)
                .setContentIntent(configureIntent)
                .build()
        )
    }
}