package org.irdest.IrdestVPN

import android.annotation.SuppressLint
import android.app.*
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.annotation.RequiresApi
import androidx.core.app.NotificationCompat
import java.io.FileInputStream
import java.io.FileOutputStream
import java.util.concurrent.atomic.AtomicReference

/* Local tun interface address*/
// IpV4
const val VPN_ADDRESS = "10.0.0.2"
const val VPN_ROUTE = "0.0.0.0"
const val VPN_DNS = "8.8.8.8"
// IpV6
const val VPN_ADDRESS_V6 = "2001:db8::1"
const val VPN_ROUTE_V6 = "::" // Intercept all

@SuppressLint("NewApi")
class IrdestVpnService : VpnService() {
    private val TAG = IrdestVpnService::class.java.simpleName

    private val ratmandThread = AtomicReference<Thread>()

    private lateinit var vpnInterface: ParcelFileDescriptor
    private lateinit var connection: Connection

    val exceptionHandler = object : Thread.UncaughtExceptionHandler {
        override fun uncaughtException(t: Thread, e: Throwable) {
            Log.e(TAG, "uncaughtExceptionHandler: ", e)
        }
    }

    companion object {
        const val ACTION_CONNECT = "IRDEST_VPN_CONNECT"
        const val ACTION_DISCONNECT = "IRDEST_VPN_DISCONNECT"
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_DISCONNECT) {
            disconnect()
            return Service.START_NOT_STICKY
        } else {
            connect()
            return Service.START_STICKY
        }
    }

    private fun connect() {
        // Become a foreground service.
        updateForegroundNotification(R.string.connecting)

        // Run `ratmand` router
        runRatmand()

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
        }
        catch (e: Exception) {
            Log.e(TAG, "connect: Failed to open tun interface", e)
        }
        finally {
            connection.connect()
            updateForegroundNotification(R.string.connected)
        }
    }

    private fun runRatmand() {
        val thread = Thread(Ratmand())
        thread.isDaemon = true
        thread.stackTrace
        thread.uncaughtExceptionHandler = exceptionHandler

        // Simply replace any existing `ratmand` thread with the new one.
        setRatmandThread(thread)
        thread.start()
    }

    private fun setRatmandThread(thr: Thread) {
        ratmandThread.getAndSet(thr)?.interrupt()
    }

    private fun disconnect() {
        ratmandThread.get().interrupt()
        connection.disconnect()
        vpnInterface.close()
        stopForeground(true)
        stopSelf()
        Log.i(TAG, "stopVPN: Vpn service is stopped")
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

    private fun updateForegroundNotification(msg: Int) {
        val NOTIFICATION_CHANNEL_ID = "IrdestVpn"
        val notificationManager = (getSystemService(NOTIFICATION_SERVICE) as NotificationManager)
        val pendintIntent = PendingIntent.getActivity(this, 0,
            Intent(this,MainActivity::class.java), PendingIntent.FLAG_UPDATE_CURRENT)

        // From Android 8.0 notification channel must be created.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            notificationManager.createNotificationChannel(
                NotificationChannel(NOTIFICATION_CHANNEL_ID, NOTIFICATION_CHANNEL_ID,
                    NotificationManager.IMPORTANCE_DEFAULT)
            )
        }

        startForeground(133, NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle("\uD83D\uDC80")
            .setContentText(getString(msg))
            .setSmallIcon(androidx.loader.R.drawable.notification_bg)
            .setContentIntent(pendintIntent)
            .build())
    }
}