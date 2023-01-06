package org.irdest.irdestVPN.utils

import android.os.Handler
import android.os.Looper
import android.os.Message

class MessageHandler(
    looper: Looper,
    private val receiver: MessageReceiver
) : Handler(looper) {

    interface MessageReceiver {
        fun onReceiveMessage(msg: Message)
    }

    override fun handleMessage(msg: Message) {
        super.handleMessage(msg)
        receiver.onReceiveMessage(msg)
    }
}