package org.irdest.IrdestVPN.utils

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import org.irdest.IrdestVPN.MainActivity
import org.irdest.IrdestVPN.R

val NOTIFICATION_ID = 1111

private val CHANNEL_ID = "IrdestVpn"
private val REQUEST_CODE = 0
private val CLIENT_ACTIVTY = MainActivity::class.java

fun updateNotification(msg: Int, context: Context) {
    NotificationManagerCompat.from(context)
        .notify(NOTIFICATION_ID, getNotification(msg, context))

}

fun getNotification(msg: Int, context: Context) : Notification {
    // Intent to be sent when the notification is clicked.
    val contentIntent = PendingIntent.getActivity(
        context,
        REQUEST_CODE,
        Intent(context, CLIENT_ACTIVTY),
        PendingIntent.FLAG_UPDATE_CURRENT
    )

    return NotificationCompat.Builder(context, CHANNEL_ID)
        .setShowWhen(false)
        .setSmallIcon(R.drawable.ic_launcher_foreground)
        .setContentIntent(contentIntent)
        .setContentText(context.getString(msg))
        .build()
}